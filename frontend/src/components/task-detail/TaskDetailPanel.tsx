import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";
import type { Session, Task, TaskStatus } from "@/api/generated/model";
import { FindingStatus } from "@/api/generated/model/findingStatus";
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
import { DiffOverlay } from "@/components/diff/DiffOverlay";
import { PhasesList } from "@/components/task-detail/PhasesList";
import { ProblemsTab } from "@/components/task-detail/ProblemsTab";
import { STATUS_CONFIG } from "@/components/kanban/KanbanColumn";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Loader } from "@/components/ui/loader";
import { Markdown } from "@/components/ui/markdown";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useSessionActivitySSE } from "@/hooks/useSessionActivitySSE";
import { cn } from "@/lib/utils";
import { useIsTaskExecuting } from "@/stores/useExecutingTasksStore";
import { CompleteTaskDialog } from "@/components/dialogs/CompleteTaskDialog";

const NEXT_STATUS: Partial<Record<TaskStatus, TaskStatus>> = {
  todo: "planning",
  planning: "in_progress",
  planning_review: "in_progress",
  in_progress: "ai_review",
  ai_review: "review",
  review: "done",
};

const EXECUTABLE_STATUSES: TaskStatus[] = [
  "todo",
  "planning",
  "in_progress",
  "ai_review",
];

interface TaskDetailPanelProps {
  task: Task;
  onClose: () => void;
}

function TaskDetailPanel({ task, onClose }: TaskDetailPanelProps) {
  const [activeTab, setActiveTab] = useState("details");
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(
    null,
  );
  const [isDiffOpen, setIsDiffOpen] = useState(false);
  const queryClient = useQueryClient();

  // Sessions are updated in real-time via SSE events - no polling needed
  const { data: sessionsData, isLoading: sessionsLoading } =
    useListSessionsForTask(task.id, {
      query: { staleTime: 30000 },
    });

  const taskSessions = sessionsData?.data ?? [];

  // Fetch plan content - always fetch so we can show link in Details
  interface PlanResponse {
    data: { content: string; exists: boolean };
    status: number;
    headers: Headers;
  }
  const { data: planData, isLoading: planLoading } = useQuery({
    queryKey: [`/api/tasks/${task.id}/plan`],
    queryFn: async () => {
      const response = await customFetch<PlanResponse>(
        `/api/tasks/${task.id}/plan`,
        {
          method: "GET",
        },
      );
      return response;
    },
    staleTime: 10000,
  });

  const hasPlan = planData?.data?.exists && planData.data.content;

  // Fetch findings - show them if they exist regardless of task status
  const { data: findingsData } = useGetTaskFindings(task.id, {
    query: {
      staleTime: 10000,
    },
  });

  const hasFindings =
    findingsData?.status === 200 &&
    findingsData.data.exists &&
    findingsData.data.findings.length > 0;

  // Count only pending findings for badge
  const pendingFindingsCount = useMemo(() => {
    if (!hasFindings || findingsData?.status !== 200) return 0;
    return findingsData.data.findings.filter(
      (f) =>
        f.status !== FindingStatus.fixed && f.status !== FindingStatus.skipped,
    ).length;
  }, [hasFindings, findingsData]);

  const latestRunningSession = useMemo(
    () => taskSessions.find((s: Session) => s.status === "running") ?? null,
    [taskSessions],
  );

  const isExecuting = useIsTaskExecuting(task.id);

  // Auto-select the latest running session when it appears
  useEffect(() => {
    if (latestRunningSession && selectedSessionId !== latestRunningSession.id) {
      // Only auto-select if user hasn't manually selected a different session
      // or if no session is currently selected
      if (
        !selectedSessionId ||
        taskSessions.find((s) => s.id === selectedSessionId)?.status !==
          "running"
      ) {
        setSelectedSessionId(latestRunningSession.id);
      }
    }
  }, [latestRunningSession, selectedSessionId, taskSessions]);

  const activeSessionId = selectedSessionId ?? latestRunningSession?.id ?? null;

  const { activities, isConnected: activityConnected } = useSessionActivitySSE(
    activeSessionId,
    {
      enabled: activeTab === "activity",
    },
  );

  const executeTask = useExecuteTask({
    mutation: {
      onSuccess: (response) => {
        void queryClient.invalidateQueries({
          queryKey: getListTasksQueryKey(),
        });
        // Invalidate sessions to pick up the new session
        void queryClient.invalidateQueries({
          queryKey: getListSessionsForTaskQueryKey(task.id),
        });
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
        void queryClient.invalidateQueries({
          queryKey: getListTasksQueryKey(),
        });
        // Auto-execute when transitioning to an executable phase
        const targetStatus = variables.data.status;
        if (EXECUTABLE_STATUSES.includes(targetStatus)) {
          executeTask.mutate({ id: task.id });
        }
      },
    },
  });

  const nextStatus = NEXT_STATUS[task.status];
  const statusConfig = STATUS_CONFIG[task.status];

  const handleExecute = () => {
    executeTask.mutate({ id: task.id });
  };

  const handleTransition = (status: TaskStatus) => {
    transitionTask.mutate({ id: task.id, data: { status } });
  };

  // Determine which action buttons to show based on status and execution state
  const renderActionButtons = () => {
    const isPending = transitionTask.isPending || executeTask.isPending;

    // If task is currently executing, show only status indicator
    if (isExecuting) {
      return (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span className="w-3 h-3 border-2 border-primary border-t-transparent rounded-full animate-spin" />
          Running {statusConfig.label}...
        </div>
      );
    }

    switch (task.status) {
      case "todo":
        // Only show "Start Planning" which transitions and auto-executes
        return (
          <Button
            size="sm"
            onClick={() => handleTransition("planning")}
            disabled={isPending}
          >
            {isPending ? (
              <>
                <span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
                Starting...
              </>
            ) : (
              "Start Planning"
            )}
          </Button>
        );

      case "planning":
      case "in_progress":
      case "ai_review":
        // Show "Run" button to re-run current phase, and "Start Next" to move forward
        return (
          <>
            <Button size="sm" onClick={handleExecute} disabled={isPending}>
              {isPending ? (
                <>
                  <span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  Running...
                </>
              ) : (
                `Run ${statusConfig.label}`
              )}
            </Button>
            {nextStatus && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => handleTransition(nextStatus)}
                disabled={isPending}
              >
                {EXECUTABLE_STATUSES.includes(nextStatus)
                  ? `Start ${STATUS_CONFIG[nextStatus].label}`
                  : `Move to ${STATUS_CONFIG[nextStatus].label}`}
              </Button>
            )}
          </>
        );

      case "planning_review":
        // Human review of plan - can approve (move to in_progress) or request changes
        return (
          <>
            <Button
              size="sm"
              onClick={() => handleTransition("in_progress")}
              disabled={isPending}
            >
              {isPending ? (
                <>
                  <span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  Starting...
                </>
              ) : (
                "Approve & Start Implementation"
              )}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => handleTransition("planning")}
              disabled={isPending}
            >
              Regenerate Plan
            </Button>
          </>
        );

      case "fix":
        // After AI Review found issues - can run fix or skip to review
        return (
          <>
            <Button size="sm" onClick={handleExecute} disabled={isPending}>
              {isPending ? (
                <>
                  <span className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
                  Fixing...
                </>
              ) : (
                "Run Fix"
              )}
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => handleTransition("review")}
              disabled={isPending}
            >
              Skip to Review
            </Button>
          </>
        );

      case "review":
        // Human review - can approve (done) via CompleteTaskDialog or request changes
        return (
          <>
            <Button
              size="sm"
              onClick={async () => {
                try {
                  const result = await CompleteTaskDialog.show({ task });
                  if (result?.success) {
                    // Task was completed successfully - invalidate queries to refresh UI
                    void queryClient.invalidateQueries({
                      queryKey: getListTasksQueryKey(),
                    });
                  }
                } catch {
                  // Dialog was cancelled - no action needed
                }
              }}
              disabled={isPending}
            >
              Approve & Complete
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => handleTransition("in_progress")}
              disabled={isPending}
            >
              Request Changes
            </Button>
          </>
        );

      case "done":
        // Completed - can reopen if needed
        return (
          <Button
            size="sm"
            variant="outline"
            onClick={() => handleTransition("in_progress")}
            disabled={isPending}
          >
            Reopen Task
          </Button>
        );

      default:
        return null;
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b px-4 py-3">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold">{task.title}</h2>
          <Badge
            className={`${statusConfig.headerBg} ${statusConfig.borderColor} border`}
          >
            <span
              className={`w-2 h-2 rounded-full ${statusConfig.dotColor} mr-1.5`}
            />
            {statusConfig.label}
          </Badge>
          {isExecuting && (
            <span className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
              Running
            </span>
          )}
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
        {renderActionButtons()}
      </div>

      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="flex-1 flex flex-col min-h-0"
      >
        <TabsList className="mx-4 mt-2 w-fit shrink-0">
          <TabsTrigger value="details">Details</TabsTrigger>
          <TabsTrigger value="activity" className="relative">
            {latestRunningSession ? (
              <>
                {latestRunningSession.implementation_phase_number
                  ? `Phase ${latestRunningSession.implementation_phase_number}`
                  : latestRunningSession.phase}
                <span className="ml-1.5 inline-block w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
              </>
            ) : (
              "Activity"
            )}
          </TabsTrigger>
          <TabsTrigger value="plan">Plan</TabsTrigger>
          {hasFindings && (
            <TabsTrigger value="problems" className="relative">
              Problems
              {pendingFindingsCount > 0 ? (
                <Badge
                  variant="destructive"
                  className="ml-1.5 h-5 px-1.5 text-xs"
                >
                  {pendingFindingsCount}
                </Badge>
              ) : (
                <Badge
                  variant="outline"
                  className="ml-1.5 h-5 px-1.5 text-xs border-green-500/50 text-green-500"
                >
                  <svg
                    className="w-3 h-3"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                </Badge>
              )}
            </TabsTrigger>
          )}
          <TabsTrigger value="sessions">Sessions</TabsTrigger>
        </TabsList>

        {/* Activity tab - needs its own scroll handling */}
        <TabsContent
          value="activity"
          className="flex-1 mt-0 min-h-0 flex flex-col"
        >
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
                  {taskSessions.map((session: Session) => {
                    // Build session label with phase info
                    let label: string = session.phase;
                    if (
                      session.implementation_phase_number &&
                      session.implementation_phase_title
                    ) {
                      label = `${session.implementation_phase_number}. ${session.implementation_phase_title}`;
                    } else if (session.implementation_phase_number) {
                      label = `Phase ${session.implementation_phase_number}`;
                    }

                    return (
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
                        title={`${session.phase} - ${session.id.slice(0, 8)}`}
                      >
                        {label}
                        {session.status === "running" && (
                          <span className="ml-1 inline-block w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
                        )}
                      </button>
                    );
                  })}
                </div>
              )}
              <ScrollArea className="flex-1">
                <ActivityFeed
                  activities={activities}
                  isConnected={activityConnected}
                  className="p-4"
                />
              </ScrollArea>
            </div>
          )}
        </TabsContent>

        {/* Details tab - contextual hub */}
        <TabsContent value="details" className="flex-1 mt-0 min-h-0">
          <ScrollArea className="h-full">
            <div className="p-4 space-y-4">
              {/* Description */}
              {task.description && (
                <p className="text-sm text-foreground/80 whitespace-pre-wrap">
                  {task.description}
                </p>
              )}

              {/* Workspace path - always visible if exists */}
              {task.workspace_path && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <span className="font-mono bg-muted/50 px-2 py-1 rounded">
                    {task.workspace_path}
                  </span>
                </div>
              )}

              {/* Plan link - show when plan exists */}
              {hasPlan && (
                <button
                  type="button"
                  onClick={() => setActiveTab("plan")}
                  className="w-full flex items-center gap-3 p-3 rounded-lg border border-border/50 bg-card/30 hover:bg-card/50 transition-colors text-left group"
                >
                  <div className="w-8 h-8 rounded-md bg-blue-500/10 flex items-center justify-center shrink-0">
                    <svg
                      className="w-4 h-4 text-blue-500/70"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                    >
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
                      <polyline points="14 2 14 8 20 8" />
                      <line x1="16" y1="13" x2="8" y2="13" />
                      <line x1="16" y1="17" x2="8" y2="17" />
                      <polyline points="10 9 9 9 8 9" />
                    </svg>
                  </div>
                  <div className="flex-1 min-w-0">
                    <span className="text-sm font-medium text-foreground/80 group-hover:text-foreground">
                      Implementation Plan
                    </span>
                    <p className="text-xs text-muted-foreground/60">
                      View generated plan
                    </p>
                  </div>
                  <svg
                    className="w-4 h-4 text-muted-foreground/40 group-hover:text-muted-foreground/60"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </button>
              )}

              {/* Findings alert - show when AI review found issues */}
              {hasFindings && findingsData?.status === 200 && (
                <button
                  type="button"
                  onClick={() => setActiveTab("problems")}
                  className={cn(
                    "w-full flex items-center gap-3 p-3 rounded-lg border transition-colors text-left group",
                    pendingFindingsCount > 0
                      ? "border-red-500/30 bg-red-500/5 hover:bg-red-500/10"
                      : "border-green-500/30 bg-green-500/5 hover:bg-green-500/10",
                  )}
                >
                  <div
                    className={cn(
                      "w-8 h-8 rounded-md flex items-center justify-center shrink-0",
                      pendingFindingsCount > 0
                        ? "bg-red-500/10"
                        : "bg-green-500/10",
                    )}
                  >
                    {pendingFindingsCount > 0 ? (
                      <svg
                        className="w-4 h-4 text-red-500/70"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                      >
                        <circle cx="12" cy="12" r="10" />
                        <line x1="12" y1="8" x2="12" y2="12" />
                        <line x1="12" y1="16" x2="12.01" y2="16" />
                      </svg>
                    ) : (
                      <svg
                        className="w-4 h-4 text-green-500/70"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                      >
                        <polyline points="20 6 9 17 4 12" />
                      </svg>
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    {pendingFindingsCount > 0 ? (
                      <>
                        <span className="text-sm font-medium text-red-500/80">
                          {pendingFindingsCount} issue
                          {pendingFindingsCount !== 1 ? "s" : ""} to fix
                        </span>
                        <p className="text-xs text-muted-foreground/60">
                          Review and fix problems
                        </p>
                      </>
                    ) : (
                      <>
                        <span className="text-sm font-medium text-green-500/80">
                          All issues fixed
                        </span>
                        <p className="text-xs text-muted-foreground/60">
                          {findingsData.data.findings.length} finding
                          {findingsData.data.findings.length !== 1 ? "s" : ""}{" "}
                          resolved
                        </p>
                      </>
                    )}
                  </div>
                  <svg
                    className="w-4 h-4 text-muted-foreground/40 group-hover:text-muted-foreground/60"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </button>
              )}

              {/* Review Changes - show when workspace exists */}
              {task.workspace_path && (
                <button
                  type="button"
                  onClick={() => setIsDiffOpen(true)}
                  className="w-full flex items-center gap-3 p-3 rounded-lg border border-border/50 bg-card/30 hover:bg-card/50 transition-colors text-left group"
                >
                  <div className="w-8 h-8 rounded-md bg-emerald-500/10 flex items-center justify-center shrink-0">
                    <svg
                      className="w-4 h-4 text-emerald-500/70"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                    >
                      <path d="M12 3v18M3 12h18" />
                    </svg>
                  </div>
                  <div className="flex-1 min-w-0">
                    <span className="text-sm font-medium text-foreground/80 group-hover:text-foreground">
                      Review Changes
                    </span>
                    <p className="text-xs text-muted-foreground/60">
                      View code diff in fullscreen
                    </p>
                  </div>
                  <svg
                    className="w-4 h-4 text-muted-foreground/40 group-hover:text-muted-foreground/60"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                </button>
              )}

              {/* Phases - show for implementation stages */}
              {["in_progress", "ai_review", "fix", "review", "done"].includes(
                task.status,
              ) && <PhasesList taskId={task.id} />}

              {/* Technical details - collapsible */}
              <details className="group">
                <summary className="flex items-center gap-2 text-xs text-muted-foreground/50 cursor-pointer hover:text-muted-foreground/70 transition-colors">
                  <svg
                    className="w-3 h-3 transition-transform group-open:rotate-90"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <polyline points="9 18 15 12 9 6" />
                  </svg>
                  Technical details
                </summary>
                <div className="mt-3 pl-5 grid grid-cols-2 gap-3 text-xs">
                  <div>
                    <span className="text-muted-foreground/50">ID</span>
                    <p className="font-mono text-muted-foreground/70">
                      {task.id.slice(0, 8)}…
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground/50">Status</span>
                    <p className="text-muted-foreground/70">
                      {statusConfig.label}
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground/50">Created</span>
                    <p className="text-muted-foreground/70">
                      {new Date(task.created_at).toLocaleString()}
                    </p>
                  </div>
                  <div>
                    <span className="text-muted-foreground/50">Updated</span>
                    <p className="text-muted-foreground/70">
                      {new Date(task.updated_at).toLocaleString()}
                    </p>
                  </div>
                </div>
              </details>
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
                  <p className="text-sm text-muted-foreground">
                    No sessions found for this task.
                  </p>
                </div>
              ) : (
                <div className="space-y-2">
                  {taskSessions.map((session) => (
                    <div
                      key={session.id}
                      className="rounded-lg border p-3 text-sm"
                    >
                      <div className="flex items-center justify-between">
                        <span className="font-medium">{session.phase}</span>
                        <Badge variant="outline">{session.status}</Badge>
                      </div>
                      <p className="mt-1 text-xs text-muted-foreground font-mono">
                        {session.opencode_session_id || "No OpenCode session"}
                      </p>
                      {session.started_at && (
                        <p className="mt-1 text-xs text-muted-foreground">
                          Started:{" "}
                          {new Date(session.started_at).toLocaleString()}
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

      {/* Fullscreen diff overlay */}
      <DiffOverlay
        taskId={task.id}
        isOpen={isDiffOpen}
        onClose={() => setIsDiffOpen(false)}
      />
    </div>
  );
}

export { TaskDetailPanel };
