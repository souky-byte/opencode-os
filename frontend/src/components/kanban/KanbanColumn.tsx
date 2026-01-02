import type { ReactNode } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";

// Consolidated column types
type KanbanColumnType =
  | "backlog"
  | "planning"
  | "in_progress"
  | "review"
  | "done";

const COLUMN_CONFIG: Record<
  KanbanColumnType,
  { label: string; headerBg: string; dotColor: string; borderColor: string }
> = {
  backlog: {
    label: "Backlog",
    headerBg: "bg-muted/50",
    dotColor: "bg-muted-foreground",
    borderColor: "border-muted-foreground/20",
  },
  planning: {
    label: "Planning",
    headerBg: "bg-blue-500/10",
    dotColor: "bg-blue-400",
    borderColor: "border-blue-500/30",
  },
  in_progress: {
    label: "In Progress",
    headerBg: "bg-amber-500/10",
    dotColor: "bg-amber-400",
    borderColor: "border-amber-500/30",
  },
  review: {
    label: "Review",
    headerBg: "bg-orange-500/10",
    dotColor: "bg-orange-400",
    borderColor: "border-orange-500/30",
  },
  done: {
    label: "Done",
    headerBg: "bg-emerald-500/10",
    dotColor: "bg-emerald-400",
    borderColor: "border-emerald-500/30",
  },
};

interface KanbanColumnProps {
  columnType: KanbanColumnType;
  count: number;
  children: ReactNode;
  onAddTask?: () => void;
}

function KanbanColumn({
  columnType,
  count,
  children,
  onAddTask,
}: KanbanColumnProps) {
  const config = COLUMN_CONFIG[columnType];

  return (
    <div
      className={cn(
        "flex h-full w-72 flex-shrink-0 flex-col rounded-xl border bg-card/30",
        config.borderColor,
      )}
    >
      <div
        className={cn(
          "flex items-center justify-between rounded-t-xl px-3 py-2.5 border-b",
          config.headerBg,
          config.borderColor,
        )}
      >
        <div className="flex items-center gap-2">
          <div className={cn("h-2 w-2 rounded-full", config.dotColor)} />
          <h3 className="text-sm font-medium text-foreground">
            {config.label}
          </h3>
          <span className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">
            {count}
          </span>
        </div>
        {onAddTask && columnType === "backlog" && (
          <button
            type="button"
            onClick={onAddTask}
            className="rounded-md p-1 text-muted-foreground hover:bg-accent hover:text-foreground transition-colors"
            aria-label="Add task"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <title>Add</title>
              <path d="M12 5v14" />
              <path d="M5 12h14" />
            </svg>
          </button>
        )}
      </div>

      <ScrollArea className="flex-1 p-2">
        <div className="flex flex-col gap-2">{children}</div>
      </ScrollArea>
    </div>
  );
}

// Legacy STATUS_CONFIG for compatibility with TaskDetailPanel
import type { TaskStatus } from "@/api/generated/model";

const STATUS_CONFIG: Record<
  TaskStatus,
  { label: string; headerBg: string; dotColor: string; borderColor: string }
> = {
  todo: {
    label: "Todo",
    headerBg: "bg-muted/50",
    dotColor: "bg-muted-foreground",
    borderColor: "border-muted-foreground/20",
  },
  planning: {
    label: "Planning",
    headerBg: "bg-blue-500/10",
    dotColor: "bg-blue-400",
    borderColor: "border-blue-500/30",
  },
  planning_review: {
    label: "Plan Review",
    headerBg: "bg-blue-500/15",
    dotColor: "bg-blue-500",
    borderColor: "border-blue-500/40",
  },
  in_progress: {
    label: "In Progress",
    headerBg: "bg-amber-500/10",
    dotColor: "bg-amber-400",
    borderColor: "border-amber-500/30",
  },
  ai_review: {
    label: "AI Review",
    headerBg: "bg-purple-500/10",
    dotColor: "bg-purple-400",
    borderColor: "border-purple-500/30",
  },
  fix: {
    label: "Fix Issues",
    headerBg: "bg-red-500/10",
    dotColor: "bg-red-400",
    borderColor: "border-red-500/30",
  },
  review: {
    label: "Review",
    headerBg: "bg-orange-500/10",
    dotColor: "bg-orange-400",
    borderColor: "border-orange-500/30",
  },
  done: {
    label: "Done",
    headerBg: "bg-emerald-500/10",
    dotColor: "bg-emerald-400",
    borderColor: "border-emerald-500/30",
  },
};

export { KanbanColumn, COLUMN_CONFIG, STATUS_CONFIG };
export type { KanbanColumnType };
