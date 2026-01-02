import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { SessionActivityMsg } from "@/types/generated/SessionActivityMsg";

function getActivityUrl(sessionId: string): string {
	const base = import.meta.env.VITE_API_URL || "";
	return `${base}/api/sessions/${sessionId}/activity`;
}

interface UseSessionActivitySSEOptions {
	enabled?: boolean;
	onActivity?: (activity: SessionActivityMsg) => void;
	onFinished?: (success: boolean, error: string | null) => void;
}

interface UseSessionActivitySSEResult {
	activities: SessionActivityMsg[];
	isConnected: boolean;
	isFinished: boolean;
	error: string | null;
	clearActivities: () => void;
}

export function useSessionActivitySSE(
	sessionId: string | null,
	options: UseSessionActivitySSEOptions = {},
): UseSessionActivitySSEResult {
	const { enabled = true, onActivity, onFinished } = options;

	const [activities, setActivities] = useState<SessionActivityMsg[]>([]);
	const [isConnected, setIsConnected] = useState(false);
	const [isFinished, setIsFinished] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const eventSourceRef = useRef<EventSource | null>(null);
	const shouldConnectRef = useRef(true);

	const onActivityRef = useRef(onActivity);
	const onFinishedRef = useRef(onFinished);

	useEffect(() => {
		onActivityRef.current = onActivity;
	}, [onActivity]);

	useEffect(() => {
		onFinishedRef.current = onFinished;
	}, [onFinished]);

	const clearActivities = useCallback(() => {
		setActivities([]);
		setIsFinished(false);
		setError(null);
	}, []);

	const handleActivity = useCallback((data: string) => {
		try {
			const activity = JSON.parse(data) as SessionActivityMsg;

			setActivities((prev) => {
				const existingIndex = prev.findIndex(
					(a) => "id" in a && "id" in activity && a.id === activity.id,
				);

				if (existingIndex >= 0) {
					const updated = [...prev];
					updated[existingIndex] = activity;
					return updated;
				}

				return [...prev, activity];
			});

			onActivityRef.current?.(activity);

			if (activity.type === "finished") {
				setIsFinished(true);
				shouldConnectRef.current = false;
				onFinishedRef.current?.(activity.success, activity.error);
			}
		} catch (e) {
			if (import.meta.env.DEV) {
				console.warn("SSE activity parse error:", e);
			}
		}
	}, []);

	useEffect(() => {
		shouldConnectRef.current = true;
		setActivities([]);
		setIsFinished(false);
		setError(null);

		if (!(sessionId && enabled)) {
			return;
		}

		const url = getActivityUrl(sessionId);
		const eventSource = new EventSource(url);
		eventSourceRef.current = eventSource;

		eventSource.onopen = () => {
			setIsConnected(true);
			setError(null);
		};

		eventSource.onerror = () => {
			setIsConnected(false);
			if (!shouldConnectRef.current) {
				eventSource.close();
			}
		};

		const activityTypes = [
			"tool_call",
			"tool_result",
			"agent_message",
			"reasoning",
			"step_start",
			"json_patch",
			"finished",
		];

		const handlers: Map<string, (e: MessageEvent<string>) => void> = new Map();
		for (const eventType of activityTypes) {
			const handler = (e: MessageEvent<string>) => handleActivity(e.data);
			handlers.set(eventType, handler);
			eventSource.addEventListener(eventType, handler);
		}

		return () => {
			shouldConnectRef.current = false;
			for (const [eventType, handler] of handlers) {
				eventSource.removeEventListener(eventType, handler);
			}
			eventSource.close();
			eventSourceRef.current = null;
		};
	}, [sessionId, enabled, handleActivity]);

	const sortedActivities = useMemo(
		() =>
			[...activities].sort((a, b) => {
				const aTime = new Date(a.timestamp).getTime();
				const bTime = new Date(b.timestamp).getTime();
				return aTime - bTime;
			}),
		[activities],
	);

	return {
		activities: sortedActivities,
		isConnected,
		isFinished,
		error,
		clearActivities,
	};
}
