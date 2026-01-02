import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useRef, useState } from "react";
import type { Session, Task, TaskStatus } from "@/api/generated/model";
import { useListSessionsForTask } from "@/api/generated/sessions/sessions";
import {
	getListTasksQueryKey,
	useExecuteTask,
	useGetTaskFindings,
	useTransitionTask,
} from "@/api/generated/tasks/tasks";
import { getListSessionsForTaskQueryKey } from "@/api/generated/sessions/sessions";
import { customFetch } from "@/lib/api-fetcher";
import { ActivityFeed } from "@/components/activity/ActivityFeed";
import { DiffViewer } from "@/components/diff";
import { PhasesList } from "@/components/task-detail/PhasesList";
import { ProblemsTab } from "@/components/task-detail/ProblemsTab";
import { STATUS_CONFIG } from "@/components/kanban/KanbanColumn";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Loader } from "@/components/ui/loader";
import { Markdown } from "@/components/ui/markdown";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useSessionActivitySSE } from "@/hooks/useSessionActivitySSE";
import { cn } from "@/lib/utils";
import { useIsTaskExecuting } from "@/stores/useExecutingTasksStore";

const NEXT_STATUS: Partial<Record<TaskStatus, TaskStatus>> = {
	todo: "planning",
	planning: "in_progress",
	planning_review: "in_progress",
	in_progress: "ai_review",
	ai_review: "review",
	review: "done",
};

const EXECUTABLE_STATUSES: TaskStatus[] = ["todo", "planning", "in_progress", "ai_review"];

interface TaskDetailPanelProps {
	task: Task;
	onClose: () => void;
}

function TaskDetailPanel({ task, onClose }: TaskDetailPanelProps) {
	const [activeTab, setActiveTab] = useState("details");
	const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);
	const queryClient = useQueryClient();

	// Sessions are updated in real-time via SSE events - no polling needed
	const { data: sessionsData, isLoading: sessionsLoading } = useListSessionsForTask(task.id, {
		query: { staleTime: 30000 },
	});

	const taskSessions = sessionsData?.data ?? [];

	// Fetch plan content
	interface PlanResponse {
		data: { content: string; exists: boolean };
		status: number;
		headers: Headers;
	}
	const { data: planData, isLoading: planLoading } = useQuery({
		queryKey: [`/api/tasks/${task.id}/plan`],
		queryFn: async () => {
			const response = await customFetch<PlanResponse>(`/api/tasks/${task.id}/plan`, {
				method: "GET",
			});
			return response;
		},
		enabled: activeTab === "plan",
		staleTime: 10000,
	});

	// Fetch findings for AI Review
	const { data: findingsData } = useGetTaskFindings(task.id, {
		query: {
			enabled: task.status === "ai_review" || task.status === "fix",
			staleTime: 10000,
		},
	});

	const hasFindings =
		findingsData?.status === 200 &&
		findingsData.data.exists &&
		findingsData.data.findings.length > 0;

	const latestRunningSession = useMemo(
		() => taskSessions.find((s: Session) => s.status === "running") ?? null,
		[taskSessions],
	);

	const isExecuting = useIsTaskExecuting(task.id);
	const prevExecutingRef = useRef(isExecuting);

	// Auto-switch to Activity tab when execution starts
	useEffect(() => {
		// Only switch when transitioning from not-executing to executing
		if (isExecuting && !prevExecutingRef.current) {
			setActiveTab("activity");
		}
		prevExecutingRef.current = isExecuting;
	}, [isExecuting]);

	// Auto-select the latest running session when it appears
	useEffect(() => {
		if (latestRunningSession && selectedSessionId !== latestRunningSession.id) {
			// Only auto-select if user hasn't manually selected a different session
			// or if no session is currently selected
			if (!selectedSessionId || taskSessions.find((s) => s.id === selectedSessionId)?.status !== "running") {
				setSelectedSessionId(latestRunningSession.id);
			}
		}
	}, [latestRunningSession, selectedSessionId, taskSessions]);

	const activeSessionId = selectedSessionId ?? latestRunningSession?.id ?? null;

	const {
		activities,
		isConnected: activityConnected,
		isFinished: activityFinished,
		error: activityError,
	} = useSessionActivitySSE(activeSessionId, {
		enabled: activeTab === "activity",
	});

	const executeTask = useExecuteTask({
		mutation: {
			onSuccess: (response) => {
				void queryClient.invalidateQueries({ queryKey: getListTasksQueryKey() });
				// Invalidate sessions to pick up the new session
				void queryClient.invalidateQueries({ queryKey: getListSessionsForTaskQueryKey(task.id) });
				// Auto-select the new session for activity streaming
				if (response.status === 202 && response.data.session_id) {
					setSelectedSessionId(response.data.session_id);
				}
			},
		},
	});

	const transitionTask = useTransitionTask({
		mutation: {
			onSuccess: (_data, variables) => {
				void queryClient.invalidateQueries({ queryKey: getListTasksQueryKey() });
				// Auto-execute when transitioning to an executable phase
				const targetStatus = variables.data.status;
				if (EXECUTABLE_STATUSES.includes(targetStatus)) {
					executeTask.mutate({ id: task.id });
				}
			},
		},
	});

	const canExecute = EXECUTABLE_STATUSES.includes(task.status);
	const nextStatus = NEXT_STATUS[task.status];
	const statusConfig = STATUS_CONFIG[task.status];

	const handleExecute = () => {
		executeTask.mutate({ id: task.id });
	};

	const handleTransition = (status: TaskStatus) => {
		transitionTask.mutate({ id: task.id, data: { status } });
	};

	return (
		<div className="flex h-full flex-col">
			<div className="flex items-center justify-between border-b px-4 py-3">
				<div className="flex items-center gap-3">
					<h2 className="text-lg font-semibold">{task.title}</h2>
					<Badge className={`${statusConfig.headerBg} ${statusConfig.borderColor} border`}>
						<span className={`w-2 h-2 rounded-full ${statusConfig.dotColor} mr-1.5`} />
						{statusConfig.label}
					</Badge>
				</div>
				<button
					type="button"
					onClick={onClose}
					className="rounded p-1 hover:bg-accent"
					aria-label="Close panel"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="20"
						height="20"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="2"
						strokeLinecap="round"
						strokeLinejoin="round"
						aria-hidden="true"
					>
						<title>Close</title>
						<path d="M18 6 6 18" />
						<path d="m6 6 12 12" />
					</svg>
				</button>
			</div>

			<div className="flex gap-2 border-b px-4 py-2">
				{task.status === "todo" && (
					<Button
						size="sm"
						onClick={() => handleTransition("planning")}
						disabled={transitionTask.isPending || executeTask.isPending}
					>
						{transitionTask.isPending || executeTask.isPending ? (
							<>
								<span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
								Starting Planning...
							</>
						) : (
							"Start Planning"
						)}
					</Button>
				)}
				{canExecute && task.status !== "todo" && (
					<Button size="sm" onClick={handleExecute} disabled={executeTask.isPending}>
						{executeTask.isPending ? (
							<>
								<span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
								Executing...
							</>
						) : (
							`Run ${STATUS_CONFIG[task.status].label}`
						)}
					</Button>
				)}
				{nextStatus && task.status !== "todo" && (
					<Button
						size="sm"
						variant="outline"
						onClick={() => handleTransition(nextStatus)}
						disabled={transitionTask.isPending || executeTask.isPending}
					>
						{EXECUTABLE_STATUSES.includes(nextStatus)
							? `Start ${STATUS_CONFIG[nextStatus].label}`
							: `Move to ${STATUS_CONFIG[nextStatus].label}`}
					</Button>
				)}
				{task.status === "review" && (
					<Button
						size="sm"
						variant="outline"
						onClick={() => handleTransition("in_progress")}
						disabled={transitionTask.isPending || executeTask.isPending}
					>
						Request Changes
					</Button>
				)}
			</div>

			<Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col min-h-0">
				<TabsList className="mx-4 mt-2 w-fit shrink-0">
					<TabsTrigger value="details">Details</TabsTrigger>
					<TabsTrigger value="activity" className="relative">
						Activity
						{latestRunningSession && (
							<span className="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-green-500 animate-pulse" />
						)}
					</TabsTrigger>
					<TabsTrigger value="plan">Plan</TabsTrigger>
					<TabsTrigger value="diff">Diff</TabsTrigger>
					{hasFindings && (
						<TabsTrigger value="problems" className="relative">
							Problems
							<Badge variant="destructive" className="ml-1.5 h-5 px-1.5 text-xs">
								{findingsData?.status === 200 ? findingsData.data.findings.length : 0}
							</Badge>
						</TabsTrigger>
					)}
					<TabsTrigger value="sessions">Sessions</TabsTrigger>
				</TabsList>

				{/* Activity tab - needs its own scroll handling */}
				<TabsContent value="activity" className="flex-1 mt-0 min-h-0 flex flex-col">
					{taskSessions.length === 0 ? (
						<div className="flex items-center justify-center flex-1 p-4">
							<p className="text-sm text-muted-foreground">
								No sessions yet. Execute a phase to start.
							</p>
						</div>
					) : (
						<div className="flex flex-col flex-1 min-h-0">
							{taskSessions.length > 1 && (
								<div className="flex gap-1 px-4 py-2 border-b overflow-x-auto shrink-0">
									{taskSessions.map((session: Session) => (
										<button
											key={session.id}
											type="button"
											onClick={() => setSelectedSessionId(session.id)}
											className={cn(
												"px-2 py-1 text-xs rounded-md shrink-0 transition-colors",
												activeSessionId === session.id
													? "bg-primary text-primary-foreground"
													: "bg-muted hover:bg-muted/80",
											)}
										>
											{session.phase}
											{session.status === "running" && (
												<span className="ml-1 inline-block w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
											)}
										</button>
									))}
								</div>
							)}
							<ScrollArea className="flex-1">
								<ActivityFeed
									activities={activities}
									isConnected={activityConnected}
									isFinished={activityFinished}
									error={activityError}
									className="p-4"
								/>
							</ScrollArea>
						</div>
					)}
				</TabsContent>

				{/* Details tab */}
				<TabsContent value="details" className="flex-1 mt-0 min-h-0">
					<ScrollArea className="h-full">
						<div className="p-4 space-y-4">
							<div>
								<h3 className="text-sm font-medium text-muted-foreground">Description</h3>
								<p className="mt-1 text-sm whitespace-pre-wrap">
									{task.description || "No description provided"}
								</p>
							</div>

							{/* Show phases for tasks in implementation or later stages */}
							{["in_progress", "ai_review", "fix", "review", "done"].includes(task.status) && (
								<>
									<Separator />
									<PhasesList taskId={task.id} />
								</>
							)}

							<Separator />

							<div className="grid grid-cols-2 gap-4 text-sm">
								<div>
									<span className="text-muted-foreground">ID</span>
									<p className="font-mono">{task.id}</p>
								</div>
								<div>
									<span className="text-muted-foreground">Status</span>
									<p>{statusConfig.label}</p>
								</div>
								<div>
									<span className="text-muted-foreground">Created</span>
									<p>{new Date(task.created_at).toLocaleString()}</p>
								</div>
								<div>
									<span className="text-muted-foreground">Updated</span>
									<p>{new Date(task.updated_at).toLocaleString()}</p>
								</div>
								{task.workspace_path && (
									<div className="col-span-2">
										<span className="text-muted-foreground">Workspace</span>
										<p className="font-mono text-xs">{task.workspace_path}</p>
									</div>
								)}
							</div>
						</div>
					</ScrollArea>
				</TabsContent>

				{/* Plan tab */}
				<TabsContent value="plan" className="flex-1 mt-0 min-h-0">
					<ScrollArea className="h-full">
						<div className="p-4">
							{planLoading ? (
								<Loader size="sm" message="Loading plan..." />
							) : planData?.data?.exists && planData.data.content ? (
								<div className="rounded-lg border bg-card p-4">
									<Markdown text={planData.data.content} />
								</div>
							) : (
								<div className="rounded-lg border bg-muted/30 p-4">
									<p className="text-sm text-muted-foreground">
										{task.status === "todo"
											? "Plan will be generated when the task enters the Planning phase."
											: task.status === "planning"
												? "Plan is being generated..."
												: "No plan available for this task."}
									</p>
								</div>
							)}
						</div>
					</ScrollArea>
				</TabsContent>

				{/* Diff tab */}
				<TabsContent value="diff" className="flex-1 mt-0 min-h-0">
					{task.workspace_path ? (
						<DiffViewer
							taskId={task.id}
							onClose={() => setActiveTab("details")}
						/>
					) : (
						<div className="flex items-center justify-center h-full">
							<div className="text-center text-muted-foreground">
								<p>No workspace associated with this task</p>
								<p className="text-sm mt-1">Start implementation to create a workspace</p>
							</div>
						</div>
					)}
				</TabsContent>

				{/* Problems tab */}
				{hasFindings && findingsData?.status === 200 && (
					<TabsContent value="problems" className="flex-1 mt-0 min-h-0">
						<ProblemsTab
							taskId={task.id}
							findings={findingsData.data.findings}
							summary={findingsData.data.summary}
						/>
					</TabsContent>
				)}

				{/* Sessions tab */}
				<TabsContent value="sessions" className="flex-1 mt-0 min-h-0">
					<ScrollArea className="h-full">
						<div className="p-4">
							{sessionsLoading ? (
								<Loader size="sm" message="Loading sessions..." />
							) : taskSessions.length === 0 ? (
								<div className="rounded-lg border bg-muted/30 p-4">
									<p className="text-sm text-muted-foreground">No sessions found for this task.</p>
								</div>
							) : (
								<div className="space-y-2">
									{taskSessions.map((session) => (
										<div key={session.id} className="rounded-lg border p-3 text-sm">
											<div className="flex items-center justify-between">
												<span className="font-medium">{session.phase}</span>
												<Badge variant="outline">{session.status}</Badge>
											</div>
											<p className="mt-1 text-xs text-muted-foreground font-mono">
												{session.opencode_session_id || "No OpenCode session"}
											</p>
											{session.started_at && (
												<p className="mt-1 text-xs text-muted-foreground">
													Started: {new Date(session.started_at).toLocaleString()}
												</p>
											)}
										</div>
									))}
								</div>
							)}
						</div>
					</ScrollArea>
				</TabsContent>
			</Tabs>
		</div>
	);
}

export { TaskDetailPanel };
