import type { Task, TaskStatus } from "@/api/generated/model";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useIsTaskExecuting } from "@/stores/useExecutingTasksStore";

// Status groupings for consolidated kanban
const AI_STATUSES: TaskStatus[] = ["planning", "in_progress", "ai_review"];
const HUMAN_STATUSES: TaskStatus[] = ["planning_review", "fix", "review"];

interface TaskCardProps {
  task: Task;
  isSelected?: boolean;
  onClick?: () => void;
}

function TaskCard({ task, isSelected, onClick }: TaskCardProps) {
  const isAiPhase = AI_STATUSES.includes(task.status);
  const isHumanPhase = HUMAN_STATUSES.includes(task.status);
  const isExecuting = useIsTaskExecuting(task.id);

  return (
    <div
      className={cn(
        "group relative rounded-lg border bg-card p-2.5 cursor-pointer",
        "transition-all duration-150 ease-out",
        "hover:bg-accent/50 hover:border-border/80",
        isSelected && "border-primary bg-primary/5 ring-1 ring-primary",
        isExecuting && "border-emerald-500/50 bg-emerald-500/5",
      )}
      onClick={onClick}
      onKeyDown={(e) => e.key === "Enter" && onClick?.()}
      tabIndex={0}
      role="button"
    >
      {isExecuting && (
        <div className="absolute -inset-px rounded-lg bg-gradient-to-r from-emerald-500/20 via-emerald-500/10 to-emerald-500/20 opacity-50 animate-pulse" />
      )}

      <div className="relative">
        <div className="flex items-start justify-between gap-2">
          <div className="flex items-center gap-1.5 min-w-0 flex-1">
            {isExecuting && (
              <span className="relative shrink-0">
                <span className="absolute inset-0 w-2 h-2 rounded-full bg-emerald-500 animate-ping opacity-75" />
                <span className="relative block w-2 h-2 rounded-full bg-emerald-500" />
              </span>
            )}
            <h4 className="text-xs font-medium leading-tight line-clamp-2 text-foreground">
              {task.title}
            </h4>
          </div>
          <div className="flex gap-1 shrink-0">
            {isAiPhase && (
              <Badge
                variant="secondary"
                className="text-[9px] px-1 py-0 h-4 bg-purple-500/10 text-purple-400 border-purple-500/20"
              >
                AI
              </Badge>
            )}
            {isHumanPhase && (
              <Badge
                variant="outline"
                className="text-[9px] px-1 py-0 h-4 bg-amber-500/10 text-amber-400 border-amber-500/20"
              >
                Human
              </Badge>
            )}
          </div>
        </div>

        {task.description && (
          <p className="mt-1.5 text-[11px] text-muted-foreground line-clamp-2 leading-relaxed">
            {task.description}
          </p>
        )}

        {isExecuting && (
          <div className="mt-2 flex items-center justify-end">
            <span className="text-emerald-400 font-medium flex items-center gap-1 text-[9px]">
              <svg
                className="w-2.5 h-2.5 animate-spin"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
              </svg>
              Running
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

export { TaskCard };
