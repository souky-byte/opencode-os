import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import type { Session, SessionPhase, SessionStatus, Task, TaskStatus } from "@/api/generated/model";
import {
	getListSessionsQueryKey,
	getListSessionsForTaskQueryKey,
	type listSessionsForTaskResponse,
} from "@/api/generated/sessions/sessions";
import { getListTasksQueryKey, type listTasksResponse } from "@/api/generated/tasks/tasks";
import { getGetTaskPhasesQueryKey } from "@/api/generated/phases/phases";
import { useExecutingTasksStore } from "@/stores/useExecutingTasksStore";
import { toast } from "@/stores/useToastStore";
import type { Event } from "@/types/generated/Event";
import type { EventEnvelope } from "@/types/generated/EventEnvelope";

const INITIAL_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;

function getEventsUrl(taskIds?: string[]): string {
	const base = import.meta.env.VITE_API_URL || "";
	const url = new URL(`${base}/api/events`, window.location.origin);
	if (taskIds && taskIds.length > 0) {
		url.searchParams.set("task_ids", taskIds.join(","));
	}
	return url.toString();
}

interface UseEventStreamOptions {
	taskId?: string;
	onEvent?: (event: Event) => void;
}

export function useEventStream(options: UseEventStreamOptions = {}) {
	const { taskId, onEvent } = options;
	const queryClient = useQueryClient();
	const startExecuting = useExecutingTasksStore((s) => s.startExecuting);
	const stopExecuting = useExecutingTasksStore((s) => s.stopExecuting);
	const eventSourceRef = useRef<EventSource | null>(null);
	const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
	const reconnectDelayRef = useRef(INITIAL_RECONNECT_DELAY);
	const shouldReconnectRef = useRef(true);
	const [isConnected, setIsConnected] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const onEventRef = useRef(onEvent);
	useEffect(() => {
		onEventRef.current = onEvent;
	}, [onEvent]);

	const updateTaskInCache = useCallback(
		(taskId: string, updates: Partial<Task>) => {
			queryClient.setQueryData<listTasksResponse>(getListTasksQueryKey(), (oldData) => {
				if (!oldData?.data) return oldData;

				const updatedTasks = oldData.data.map((task) =>
					task.id === taskId
						? { ...task, ...updates, updated_at: new Date().toISOString() }
						: task,
				);

				return { ...oldData, data: updatedTasks };
			});
		},
		[queryClient],
	);

	// Add a new session to the cache for a specific task
	const addSessionToCache = useCallback(
		(taskId: string, session: Session) => {
			// Update per-task sessions cache
			queryClient.setQueryData<listSessionsForTaskResponse>(
				getListSessionsForTaskQueryKey(taskId),
				(oldData) => {
					if (!oldData?.data) {
						return { data: [session], status: 200, headers: new Headers() } as listSessionsForTaskResponse;
					}
					// Don't add if already exists
					if (oldData.data.some((s) => s.id === session.id)) {
						return oldData;
					}
					return { ...oldData, data: [...oldData.data, session] };
				},
			);
			// Also invalidate the global sessions list
			void queryClient.invalidateQueries({ queryKey: getListSessionsQueryKey() });
		},
		[queryClient],
	);

	// Update a session's status in the cache
	const updateSessionInCache = useCallback(
		(taskId: string, sessionId: string, updates: Partial<Session>) => {
			queryClient.setQueryData<listSessionsForTaskResponse>(
				getListSessionsForTaskQueryKey(taskId),
				(oldData) => {
					if (!oldData?.data) return oldData;
					return {
						...oldData,
						data: oldData.data.map((session) =>
							session.id === sessionId ? { ...session, ...updates } : session,
						),
					};
				},
			);
		},
		[queryClient],
	);

	const invalidateQueries = useCallback(
		(event: Event) => {
			switch (event.type) {
				case "task.created":
					// New task - need full refetch to get all data
					void queryClient.invalidateQueries({ queryKey: getListTasksQueryKey() });
					break;
				case "task.updated":
					// Task update without status change - refetch to get all updates
					void queryClient.invalidateQueries({ queryKey: getListTasksQueryKey() });
					break;
				case "task.status_changed":
					// Optimistic update for status change - immediate UI feedback
					updateTaskInCache(event.task_id, {
						status: event.to_status as TaskStatus,
					});
					break;
				case "session.started": {
					// Mark task as executing
					startExecuting(event.task_id);
					// Immediately add session to cache using enriched event data
					const newSession: Session = {
						id: event.session_id,
						task_id: event.task_id,
						phase: event.phase as SessionPhase,
						status: event.status as SessionStatus,
						opencode_session_id: event.opencode_session_id ?? undefined,
						created_at: event.created_at,
						started_at: new Date().toISOString(),
					};
					addSessionToCache(event.task_id, newSession);
					// Show notification
					toast.info(`${event.phase} session started`);
					break;
				}
				case "session.ended": {
					// Mark task as no longer executing
					stopExecuting(event.task_id);
					// Update session status in cache
					updateSessionInCache(event.task_id, event.session_id, {
						status: (event.success ? "completed" : "failed") as SessionStatus,
						completed_at: new Date().toISOString(),
					});
					// Also invalidate for full refresh
					void queryClient.invalidateQueries({ queryKey: getListSessionsQueryKey() });
					// Show notification
					if (event.success) {
						toast.success("Session completed successfully");
					} else {
						toast.error("Session failed");
					}
					break;
				}
				case "workspace.created":
				case "workspace.merged":
				case "workspace.deleted":
					void queryClient.invalidateQueries({ queryKey: getListTasksQueryKey() });
					break;
				case "phase.completed": {
					// Invalidate phases query to update the UI
					void queryClient.invalidateQueries({ queryKey: getGetTaskPhasesQueryKey(event.task_id) });
					// Also invalidate sessions as a new session may have been created
					void queryClient.invalidateQueries({ queryKey: getListSessionsForTaskQueryKey(event.task_id) });
					toast.success(`Phase ${event.phase_number}/${event.total_phases} completed: ${event.phase_title}`);
					break;
				}
				case "phase.continuing": {
					// Invalidate phases query
					void queryClient.invalidateQueries({ queryKey: getGetTaskPhasesQueryKey(event.task_id) });
					toast.info(`Starting phase ${event.next_phase_number}/${event.total_phases}`);
					break;
				}
			}
		},
		[queryClient, updateTaskInCache, addSessionToCache, updateSessionInCache, startExecuting, stopExecuting],
	);

	const handleEvent = useCallback(
		(messageEvent: MessageEvent<string>) => {
			try {
				const envelope = JSON.parse(messageEvent.data) as EventEnvelope;
				invalidateQueries(envelope.event);
				onEventRef.current?.(envelope.event);
			} catch (e) {
				if (import.meta.env.DEV) {
					console.warn("SSE parse error:", e);
				}
			}
		},
		[invalidateQueries],
	);

	useEffect(() => {
		shouldReconnectRef.current = true;
		reconnectDelayRef.current = INITIAL_RECONNECT_DELAY;

		const connect = () => {
			if (eventSourceRef.current?.readyState === EventSource.OPEN) {
				return;
			}

			const taskIds = taskId ? [taskId] : undefined;
			const url = getEventsUrl(taskIds);

			const eventSource = new EventSource(url);
			eventSourceRef.current = eventSource;

			eventSource.onopen = () => {
				setIsConnected(true);
				setError(null);
				reconnectDelayRef.current = INITIAL_RECONNECT_DELAY;
			};

			eventSource.onerror = () => {
				setIsConnected(false);
				setError("Error");
				eventSource.close();
				eventSourceRef.current = null;

				if (shouldReconnectRef.current) {
					reconnectTimeoutRef.current = setTimeout(() => {
						reconnectDelayRef.current = Math.min(
							reconnectDelayRef.current * 1.5,
							MAX_RECONNECT_DELAY,
						);
						connect();
					}, reconnectDelayRef.current);
				}
			};

			const eventTypes = [
				"task.created",
				"task.updated",
				"task.status_changed",
				"session.started",
				"session.ended",
				"phase.completed",
				"phase.continuing",
				"agent.message",
				"tool.execution",
				"workspace.created",
				"workspace.merged",
				"workspace.deleted",
				"project.opened",
				"project.closed",
				"error",
			];

			for (const eventType of eventTypes) {
				eventSource.addEventListener(eventType, handleEvent);
			}
		};

		connect();

		return () => {
			shouldReconnectRef.current = false;
			if (reconnectTimeoutRef.current) {
				clearTimeout(reconnectTimeoutRef.current);
				reconnectTimeoutRef.current = null;
			}
			if (eventSourceRef.current) {
				eventSourceRef.current.close();
				eventSourceRef.current = null;
			}
		};
	}, [taskId, handleEvent]);

	return {
		isConnected,
		error,
	};
}
