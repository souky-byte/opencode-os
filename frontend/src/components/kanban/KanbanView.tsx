import { useMemo } from "react";
import type { Task, TaskStatus } from "@/api/generated/model";
import { useListTasks } from "@/api/generated/tasks/tasks";
import { Loader } from "@/components/ui/loader";
import { ScrollArea } from "@/components/ui/scroll-area";
import { KanbanColumn } from "./KanbanColumn";
import type { KanbanColumnType } from "./KanbanColumn";
import { TaskCard } from "./TaskCard";

// Map original statuses to consolidated columns
const STATUS_TO_COLUMN: Record<TaskStatus, KanbanColumnType> = {
  todo: "backlog",
  planning: "planning",
  planning_review: "planning",
  in_progress: "in_progress",
  ai_review: "in_progress",
  fix: "in_progress",
  review: "review",
  done: "done",
};

// Sub-groups within columns for visual breakdown
type SubGroup = {
  key: string;
  label: string;
  statuses: TaskStatus[];
};

const COLUMN_SUBGROUPS: Record<KanbanColumnType, SubGroup[]> = {
  backlog: [{ key: "all", label: "", statuses: ["todo"] }],
  planning: [
    { key: "ai", label: "AI Planning", statuses: ["planning"] },
    { key: "review", label: "Awaiting Review", statuses: ["planning_review"] },
  ],
  in_progress: [
    { key: "ai", label: "AI Working", statuses: ["in_progress"] },
    { key: "ai_review", label: "AI Review", statuses: ["ai_review"] },
    { key: "fix", label: "Needs Fix", statuses: ["fix"] },
  ],
  review: [{ key: "all", label: "", statuses: ["review"] }],
  done: [{ key: "all", label: "", statuses: ["done"] }],
};

const COLUMN_ORDER: KanbanColumnType[] = [
  "backlog",
  "planning",
  "in_progress",
  "review",
  "done",
];

interface KanbanViewProps {
  selectedTaskId?: string;
  onSelectTask: (task: Task) => void;
  onAddTask: () => void;
}

function KanbanView({
  selectedTaskId,
  onSelectTask,
  onAddTask,
}: KanbanViewProps) {
  const { data, isLoading, error } = useListTasks();

  const tasksByColumn = useMemo(() => {
    const tasks = data?.data ?? [];
    const grouped: Record<KanbanColumnType, Task[]> = {
      backlog: [],
      planning: [],
      in_progress: [],
      review: [],
      done: [],
    };

    for (const task of tasks) {
      const column = STATUS_TO_COLUMN[task.status];
      if (column) {
        grouped[column].push(task);
      }
    }

    return grouped;
  }, [data]);

  const renderColumnContent = (columnType: KanbanColumnType, tasks: Task[]) => {
    const subgroups = COLUMN_SUBGROUPS[columnType];

    // Single group without label - just render cards
    if (subgroups.length === 1 && !subgroups[0].label) {
      return tasks.map((task) => (
        <TaskCard
          key={task.id}
          task={task}
          isSelected={task.id === selectedTaskId}
          onClick={() => onSelectTask(task)}
        />
      ));
    }

    // Multiple subgroups - render with subtle separators
    const result: React.ReactNode[] = [];

    for (const subgroup of subgroups) {
      const subgroupTasks = tasks.filter((t) =>
        subgroup.statuses.includes(t.status),
      );

      if (subgroupTasks.length === 0) continue;

      // Add separator with label
      if (result.length > 0) {
        result.push(
          <div
            key={`sep-${subgroup.key}`}
            className="flex items-center gap-2 py-1.5 mt-1"
          >
            <div className="flex-1 h-px bg-border/40" />
            <span className="text-[9px] text-muted-foreground/50 uppercase tracking-wider font-medium">
              {subgroup.label}
            </span>
            <div className="flex-1 h-px bg-border/40" />
          </div>,
        );
      } else if (subgroup.label) {
        // First group with label
        result.push(
          <div
            key={`label-${subgroup.key}`}
            className="flex items-center gap-2 pb-1.5"
          >
            <span className="text-[9px] text-muted-foreground/50 uppercase tracking-wider font-medium">
              {subgroup.label}
            </span>
            <div className="flex-1 h-px bg-border/40" />
          </div>,
        );
      }

      // Add tasks
      for (const task of subgroupTasks) {
        result.push(
          <TaskCard
            key={task.id}
            task={task}
            isSelected={task.id === selectedTaskId}
            onClick={() => onSelectTask(task)}
          />,
        );
      }
    }

    return result;
  };

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
          <p className="text-sm text-muted-foreground">
            Please try again later
          </p>
        </div>
      </div>
    );
  }

  return (
    <ScrollArea orientation="horizontal" className="h-full">
      <div className="flex h-full gap-3 p-3">
        {COLUMN_ORDER.map((columnType) => (
          <KanbanColumn
            key={columnType}
            columnType={columnType}
            count={tasksByColumn[columnType].length}
            onAddTask={columnType === "backlog" ? onAddTask : undefined}
          >
            {renderColumnContent(columnType, tasksByColumn[columnType])}
          </KanbanColumn>
        ))}
      </div>
    </ScrollArea>
  );
}

export { KanbanView };
