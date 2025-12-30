"use client";

import { useState } from "react";
import { Sidebar } from "./sidebar";
import { Header } from "./header";
import { KanbanView } from "./kanban/kanban-view";
import { RoadmapView } from "./roadmap/roadmap-view";
import { SessionsView } from "./sessions/sessions-view";
import { SettingsView } from "./settings/settings-view";
import { TaskDetailPanel } from "./task-detail/task-detail-panel";
import { mockTasks, mockProjects, type Task, type Project } from "~/lib/mock-data";

export type View = "kanban" | "roadmap" | "sessions" | "settings";

export function AppShell() {
	const [activeView, setActiveView] = useState<View>("kanban");
	const [selectedTask, setSelectedTask] = useState<Task | null>(null);
	const [tasks, setTasks] = useState(mockTasks);
	const [selectedProject, setSelectedProject] = useState<Project>(() => mockProjects[0]!);
	const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

	const handleTaskSelect = (task: Task) => {
		setSelectedTask(task);
	};

	const handleCloseTaskDetail = () => {
		setSelectedTask(null);
	};

	const handleTaskStatusChange = (taskId: string, newStatus: Task["status"]) => {
		setTasks((prev) =>
			prev.map((t) =>
				t.id === taskId ? { ...t, status: newStatus, updatedAt: new Date().toISOString() } : t,
			),
		);
		if (selectedTask?.id === taskId) {
			setSelectedTask((prev) => (prev ? { ...prev, status: newStatus } : null));
		}
	};

	const handleProjectChange = (project: Project) => {
		setSelectedProject(project);
		setSelectedTask(null); // Close any open task detail when switching projects
	};

	return (
		<div className="flex h-screen bg-background">
			<Sidebar
				activeView={activeView}
				onViewChange={setActiveView}
				projects={mockProjects}
				selectedProject={selectedProject}
				onProjectChange={handleProjectChange}
				collapsed={sidebarCollapsed}
				onCollapsedChange={setSidebarCollapsed}
			/>
			<div className="flex flex-1 flex-col overflow-hidden">
				<Header project={selectedProject} />
				<main className="flex flex-1 overflow-hidden">
					<div className={`flex-1 overflow-auto ${selectedTask ? "mr-[480px]" : ""}`}>
						{activeView === "kanban" && (
							<KanbanView
								tasks={tasks}
								onTaskSelect={handleTaskSelect}
								onTaskStatusChange={handleTaskStatusChange}
							/>
						)}
						{activeView === "roadmap" && <RoadmapView />}
						{activeView === "sessions" && <SessionsView />}
						{activeView === "settings" && <SettingsView />}
					</div>
					{selectedTask && (
						<TaskDetailPanel
							task={selectedTask}
							onClose={handleCloseTaskDetail}
							onStatusChange={handleTaskStatusChange}
						/>
					)}
				</main>
			</div>
		</div>
	);
}
