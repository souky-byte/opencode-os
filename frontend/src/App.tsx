import { useCallback, useEffect, useMemo, useState } from "react";
import type { Task } from "@/api/generated/model";
import { useGetCurrentProject } from "@/api/generated/projects/projects";
import { useExecuteTask, useListTasks } from "@/api/generated/tasks/tasks";
import { CreateTaskDialog } from "@/components/dialogs/CreateTaskDialog";
import { ProjectPickerDialog } from "@/components/dialogs/ProjectPickerDialog";
import { GitHubSettings } from "@/components/settings/GitHubSettings";
import { ModelSettings } from "@/components/settings/ModelSettings";
import { KanbanView } from "@/components/kanban/KanbanView";
import { SessionsList } from "@/components/sessions/SessionsList";
import { PullRequestsView } from "@/components/pull-requests";
import { TaskDetailPanel } from "@/components/task-detail/TaskDetailPanel";
import { Badge } from "@/components/ui/badge";
import { Loader } from "@/components/ui/loader";
import { ToastContainer } from "@/components/ui/toast";
import { useEventStream } from "@/hooks/useEventStream";
import { cn } from "@/lib/utils";
import { useDiffViewerStore } from "@/stores/useDiffViewerStore";
import { useProjectStore } from "@/stores/useProjectStore";
import { useSidebarStore } from "@/stores/useSidebarStore";

// Navigation icons
const icons = {
  kanban: (
    <svg
      className="w-5 h-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <rect x="3" y="3" width="5" height="18" rx="1" />
      <rect x="10" y="3" width="5" height="12" rx="1" />
      <rect x="17" y="3" width="5" height="8" rx="1" />
    </svg>
  ),
  pull_requests: (
    <svg
      className="w-5 h-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <circle cx="6" cy="6" r="3" />
      <circle cx="6" cy="18" r="3" />
      <circle cx="18" cy="18" r="3" />
      <path d="M6 9v6" />
      <path d="M18 9a3 3 0 00-3-3h-4" />
      <path d="M18 15v-6" />
    </svg>
  ),
  sessions: (
    <svg
      className="w-5 h-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path d="M12 8v4l3 3" />
      <circle cx="12" cy="12" r="9" />
    </svg>
  ),
  settings: (
    <svg
      className="w-5 h-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <circle cx="12" cy="12" r="3" />
      <path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" />
    </svg>
  ),
  folder: (
    <svg
      className="w-4 h-4"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
    >
      <path d="M3 7v13a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
    </svg>
  ),
  chevronLeft: (
    <svg
      className="w-4 h-4"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <path d="M15 18l-6-6 6-6" />
    </svg>
  ),
  chevronRight: (
    <svg
      className="w-4 h-4"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <path d="M9 18l6-6-6-6" />
    </svg>
  ),
};

export default function App() {
  const { activeView, setActiveView, collapsed, toggleCollapsed } =
    useSidebarStore();
  const {
    currentProject,
    isLoading: isProjectLoading,
    setCurrentProject,
    setLoading,
    openDialog,
  } = useProjectStore();
  const { isExpanded: isDiffExpanded } = useDiffViewerStore();
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  // Fetch tasks to get the latest task data (reactive to SSE updates)
  const { data: tasksData } = useListTasks();

  // Get the selected task from the tasks list - this makes it reactive to updates
  const selectedTask = useMemo(() => {
    if (!selectedTaskId || !tasksData?.data) return null;
    return tasksData.data.find((t) => t.id === selectedTaskId) ?? null;
  }, [selectedTaskId, tasksData]);

  // Clear selection when project changes (tasks will be different)
  const prevProjectPath = useMemo(() => currentProject?.path, [currentProject]);
  useEffect(() => {
    // Clear task selection when project changes
    setSelectedTaskId(null);
  }, [prevProjectPath]);

  // Auto-execute task when AI Review is triggered
  const executeTask = useExecuteTask();
  const handleAutoExecute = useCallback(
    (taskId: string) => {
      executeTask.mutate({ id: taskId });
    },
    [executeTask],
  );

  const { isConnected } = useEventStream({
    taskId: selectedTask?.id,
    onAutoExecute: handleAutoExecute,
  });

  const { data: currentProjectResponse, isLoading: isFetchingProject } =
    useGetCurrentProject({
      query: {
        retry: false,
      },
    });

  useEffect(() => {
    if (!isFetchingProject) {
      const project = currentProjectResponse?.data.project ?? null;
      setCurrentProject(project);
      setLoading(false);

      if (!project) {
        void ProjectPickerDialog.show({ allowClose: false });
      }
    }
  }, [
    isFetchingProject,
    currentProjectResponse,
    setCurrentProject,
    setLoading,
  ]);

  const handleSelectTask = (task: Task) => {
    setSelectedTaskId(task.id);
  };

  const handleClosePanel = useCallback(() => {
    setSelectedTaskId(null);
  }, []);

  // Global Escape key handler to close panel
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape" && selectedTaskId) {
        handleClosePanel();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [selectedTaskId, handleClosePanel]);

  // Cmd+B to toggle sidebar
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "b") {
        e.preventDefault();
        toggleCollapsed();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [toggleCollapsed]);

  const handleAddTask = () => {
    void CreateTaskDialog.show({});
  };

  const handleSwitchProject = () => {
    openDialog();
    void ProjectPickerDialog.show({ allowClose: true });
  };

  if (isProjectLoading || isFetchingProject) {
    return (
      <div className="flex h-screen items-center justify-center bg-background dark">
        <div className="flex flex-col items-center gap-4">
          <Loader />
          <span className="text-sm text-muted-foreground">
            Loading project...
          </span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-background">
      <ToastContainer />
      <aside
        className={`flex flex-col border-r border-border bg-card/50 transition-all duration-200 ${
          collapsed ? "w-12" : "w-48"
        }`}
      >
        {/* Header with project info */}
        <div className="flex h-11 items-center border-b border-border px-2">
          {!collapsed ? (
            <button
              type="button"
              onClick={handleSwitchProject}
              className="flex flex-1 items-center gap-2.5 rounded-md px-2 py-1.5 hover:bg-accent transition-colors min-w-0"
            >
              <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary/10 text-primary">
                {icons.folder}
              </div>
              <div className="flex flex-col min-w-0 text-left">
                <span className="text-sm font-medium truncate">
                  {currentProject?.name ?? "No Project"}
                </span>
                {currentProject && (
                  <span className="text-[11px] text-muted-foreground truncate">
                    {currentProject.vcs}
                  </span>
                )}
              </div>
            </button>
          ) : (
            <button
              type="button"
              onClick={handleSwitchProject}
              className="flex h-8 w-8 mx-auto items-center justify-center rounded-md bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
            >
              {icons.folder}
            </button>
          )}
          <button
            type="button"
            onClick={toggleCollapsed}
            className="ml-auto shrink-0 rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
          >
            {collapsed ? icons.chevronRight : icons.chevronLeft}
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-1.5 space-y-0.5">
          {(["kanban", "pull_requests", "sessions", "settings"] as const).map(
            (view) => (
              <button
                key={view}
                type="button"
                onClick={() => setActiveView(view)}
                className={`flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs font-medium transition-colors ${
                  activeView === view
                    ? "bg-primary/10 text-primary"
                    : "text-muted-foreground hover:bg-accent hover:text-foreground"
                } ${collapsed ? "justify-center" : ""}`}
              >
                <span className={activeView === view ? "text-primary" : ""}>
                  {icons[view]}
                </span>
                {!collapsed && (
                  <span className="capitalize truncate">
                    {view === "pull_requests" ? "PRs" : view}
                  </span>
                )}
              </button>
            ),
          )}
        </nav>

        {/* Connection status */}
        <div className="border-t border-border p-2">
          <div
            className={`flex items-center gap-2 ${collapsed ? "justify-center" : "px-1.5"}`}
          >
            <div className="relative">
              <div
                className={`h-1.5 w-1.5 rounded-full ${isConnected ? "bg-green-500" : "bg-red-500"}`}
              />
              {isConnected && (
                <div className="absolute inset-0 h-1.5 w-1.5 rounded-full bg-green-500 animate-ping opacity-75" />
              )}
            </div>
            {!collapsed && (
              <span className="text-[10px] text-muted-foreground">
                {isConnected ? "Connected" : "Disconnected"}
              </span>
            )}
          </div>
        </div>
      </aside>

      <main className="flex flex-1 overflow-hidden">
        {activeView === "kanban" && (
          <>
            {/* Hide kanban when diff viewer is expanded */}
            {!isDiffExpanded && (
              <div
                className={cn(
                  "flex-1 overflow-hidden transition-all duration-200",
                  selectedTask ? "hidden lg:block" : "",
                )}
              >
                <KanbanView
                  selectedTaskId={selectedTask?.id}
                  onSelectTask={handleSelectTask}
                  onAddTask={handleAddTask}
                />
              </div>
            )}

            {selectedTask && (
              <>
                {/* Backdrop overlay - visible on mobile/tablet when panel is open */}
                <div
                  className="fixed inset-0 bg-background/80 backdrop-blur-sm lg:hidden z-40"
                  onClick={handleClosePanel}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") handleClosePanel();
                  }}
                  role="button"
                  tabIndex={0}
                  aria-label="Close panel"
                />
                <div
                  className={cn(
                    "border-l border-border bg-card transition-all duration-200 z-50",
                    isDiffExpanded
                      ? "w-full"
                      : "w-full lg:w-[480px] xl:w-[560px]",
                    // On mobile, make it a fixed overlay
                    "fixed lg:relative inset-0 lg:inset-auto",
                  )}
                >
                  <TaskDetailPanel
                    task={selectedTask}
                    onClose={handleClosePanel}
                  />
                </div>
              </>
            )}
          </>
        )}

        {activeView === "pull_requests" && (
          <div className="flex-1 overflow-hidden">
            <PullRequestsView />
          </div>
        )}

        {activeView === "sessions" && (
          <div className="flex-1 overflow-auto p-6">
            <div className="mx-auto max-w-5xl">
              <h1 className="text-2xl font-bold">Sessions</h1>
              <p className="mt-2 text-muted-foreground">
                View active and past AI sessions.
              </p>
              <div className="mt-6">
                <SessionsList />
              </div>
            </div>
          </div>
        )}

        {activeView === "settings" && (
          <div className="flex-1 overflow-auto p-6">
            <div className="mx-auto max-w-4xl">
              <h1 className="text-2xl font-bold">Settings</h1>
              <p className="mt-2 text-muted-foreground">
                Configure your preferences.
              </p>

              <div className="mt-6 space-y-4">
                <div className="rounded-lg border p-4">
                  <h3 className="font-medium">Connection Status</h3>
                  <div className="mt-2 flex items-center gap-2">
                    <Badge variant={isConnected ? "success" : "destructive"}>
                      {isConnected ? "Connected" : "Disconnected"}
                    </Badge>
                    <span className="text-sm text-muted-foreground">
                      WebSocket connection to backend
                    </span>
                  </div>
                </div>

                <GitHubSettings />
                <ModelSettings />
              </div>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
