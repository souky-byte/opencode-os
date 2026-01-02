import { useMemo } from "react";
import type { Task, TaskStatus } from "@/api/generated/model";
import { useListTasks } from "@/api/generated/tasks/tasks";
import { Loader } from "@/components/ui/loader";
import { ScrollArea } from "@/components/ui/scroll-area";
import { KanbanColumn } from "./KanbanColumn";
import { TaskCard } from "./TaskCard";

const COLUMN_ORDER: TaskStatus[] = [
	"todo",
	"planning",
	"planning_review",
	"in_progress",
	"ai_review",
	"review",
	"done",
];

interface KanbanViewProps {
	selectedTaskId?: string;
	onSelectTask: (task: Task) => void;
	onAddTask: () => void;
}

function KanbanView({ selectedTaskId, onSelectTask, onAddTask }: KanbanViewProps) {
	const { data, isLoading, error } = useListTasks();

	const tasksByStatus = useMemo(() => {
		const tasks = data?.data ?? [];
		const grouped: Record<TaskStatus, Task[]> = {
			todo: [],
			planning: [],
			planning_review: [],
			in_progress: [],
			ai_review: [],
			review: [],
			done: [],
		};

		for (const task of tasks) {
			if (grouped[task.status]) {
				grouped[task.status].push(task);
			}
		}

		return grouped;
	}, [data]);

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<Loader message="Loading tasks..." />
			</div>
		);
	}

	if (error) {
		return (
			<div className="flex h-full items-center justify-center">
				<div className="text-center">
					<p className="text-destructive">Failed to load tasks</p>
					<p className="text-sm text-muted-foreground">Please try again later</p>
				</div>
			</div>
		);
	}

	return (
		<ScrollArea orientation="horizontal" className="h-full">
			<div className="flex h-full gap-4 p-4">
				{COLUMN_ORDER.map((status) => (
					<KanbanColumn
						key={status}
						status={status}
						count={tasksByStatus[status].length}
						onAddTask={status === "todo" ? onAddTask : undefined}
					>
						{tasksByStatus[status].map((task) => (
							<TaskCard
								key={task.id}
								task={task}
								isSelected={task.id === selectedTaskId}
								onClick={() => onSelectTask(task)}
							/>
						))}
					</KanbanColumn>
				))}
			</div>
		</ScrollArea>
	);
}

export { KanbanView };
