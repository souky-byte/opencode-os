import { useEffect, useState } from "react";
import type { Task } from "@/api/generated/model";
import { useGetCurrentProject } from "@/api/generated/projects/projects";
import { CreateTaskDialog } from "@/components/dialogs/CreateTaskDialog";
import { ProjectPickerDialog } from "@/components/dialogs/ProjectPickerDialog";
import { KanbanView } from "@/components/kanban/KanbanView";
import { SessionsList } from "@/components/sessions/SessionsList";
import { TaskDetailPanel } from "@/components/task-detail/TaskDetailPanel";
import { Badge } from "@/components/ui/badge";
import { Loader } from "@/components/ui/loader";
import { ToastContainer } from "@/components/ui/toast";
import { useEventStream } from "@/hooks/useEventStream";
import { useProjectStore } from "@/stores/useProjectStore";
import { useSidebarStore } from "@/stores/useSidebarStore";

// Navigation icons
const icons = {
	kanban: (
		<svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
			<rect x="3" y="3" width="5" height="18" rx="1" />
			<rect x="10" y="3" width="5" height="12" rx="1" />
			<rect x="17" y="3" width="5" height="8" rx="1" />
		</svg>
	),
	sessions: (
		<svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
			<path d="M12 8v4l3 3" />
			<circle cx="12" cy="12" r="9" />
		</svg>
	),
	settings: (
		<svg className="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
			<circle cx="12" cy="12" r="3" />
			<path d="M12 1v4M12 19v4M4.22 4.22l2.83 2.83M16.95 16.95l2.83 2.83M1 12h4M19 12h4M4.22 19.78l2.83-2.83M16.95 7.05l2.83-2.83" />
		</svg>
	),
	folder: (
		<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
			<path d="M3 7v13a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
		</svg>
	),
	chevronLeft: (
		<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
			<path d="M15 18l-6-6 6-6" />
		</svg>
	),
	chevronRight: (
		<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
			<path d="M9 18l6-6-6-6" />
		</svg>
	),
};

export default function App() {
	const { activeView, setActiveView, collapsed, toggleCollapsed } = useSidebarStore();
	const {
		currentProject,
		isLoading: isProjectLoading,
		setCurrentProject,
		setLoading,
		openDialog,
	} = useProjectStore();
	const [selectedTask, setSelectedTask] = useState<Task | null>(null);
	const { isConnected } = useEventStream({
		taskId: selectedTask?.id,
	});

	const { data: currentProjectResponse, isLoading: isFetchingProject } = useGetCurrentProject({
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
	}, [isFetchingProject, currentProjectResponse, setCurrentProject, setLoading]);

	const handleSelectTask = (task: Task) => {
		setSelectedTask(task);
	};

	const handleClosePanel = () => {
		setSelectedTask(null);
	};

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
					<span className="text-sm text-muted-foreground">Loading project...</span>
				</div>
			</div>
		);
	}

	return (
		<div className="flex h-screen bg-background">
			<ToastContainer />
			<aside
				className={`flex flex-col border-r border-border bg-card/50 transition-all duration-200 ${
					collapsed ? "w-16" : "w-60"
				}`}
			>
				{/* Header with project info */}
				<div className="flex h-14 items-center border-b border-border px-3">
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
									<span className="text-[11px] text-muted-foreground truncate">{currentProject.vcs}</span>
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
				<nav className="flex-1 p-2 space-y-1">
					{(["kanban", "sessions", "settings"] as const).map((view) => (
						<button
							key={view}
							type="button"
							onClick={() => setActiveView(view)}
							className={`flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-sm font-medium transition-colors ${
								activeView === view
									? "bg-primary/10 text-primary"
									: "text-muted-foreground hover:bg-accent hover:text-foreground"
							} ${collapsed ? "justify-center" : ""}`}
						>
							<span className={activeView === view ? "text-primary" : ""}>
								{icons[view]}
							</span>
							{!collapsed && <span className="capitalize">{view}</span>}
						</button>
					))}
				</nav>

				{/* Connection status */}
				<div className="border-t border-border p-3">
					<div className={`flex items-center gap-2.5 ${collapsed ? "justify-center" : "px-2"}`}>
						<div className="relative">
							<div
								className={`h-2 w-2 rounded-full ${isConnected ? "bg-green-500" : "bg-red-500"}`}
							/>
							{isConnected && (
								<div className="absolute inset-0 h-2 w-2 rounded-full bg-green-500 animate-ping opacity-75" />
							)}
						</div>
						{!collapsed && (
							<span className="text-xs text-muted-foreground">
								{isConnected ? "Connected" : "Disconnected"}
							</span>
						)}
					</div>
				</div>
			</aside>

			<main className="flex flex-1 overflow-hidden">
				{activeView === "kanban" && (
					<>
						<div className={`flex-1 overflow-hidden ${selectedTask ? "hidden lg:block" : ""}`}>
							<KanbanView
								selectedTaskId={selectedTask?.id}
								onSelectTask={handleSelectTask}
								onAddTask={handleAddTask}
							/>
						</div>

						{selectedTask && (
							<div className="w-full lg:w-[480px] xl:w-[560px] border-l border-border bg-card">
								<TaskDetailPanel task={selectedTask} onClose={handleClosePanel} />
							</div>
						)}
					</>
				)}

				{activeView === "sessions" && (
					<div className="flex-1 overflow-auto p-6">
						<div className="mx-auto max-w-5xl">
							<h1 className="text-2xl font-bold">Sessions</h1>
							<p className="mt-2 text-muted-foreground">View active and past AI sessions.</p>
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
							<p className="mt-2 text-muted-foreground">Configure your preferences.</p>

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
							</div>
						</div>
					</div>
				)}
			</main>
		</div>
	);
}
