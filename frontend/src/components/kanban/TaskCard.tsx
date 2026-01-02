import type { Task, TaskStatus } from "@/api/generated/model";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { useIsTaskExecuting } from "@/stores/useExecutingTasksStore";

const AI_PHASES: TaskStatus[] = ["planning", "in_progress", "ai_review"];
const HUMAN_PHASES: TaskStatus[] = ["planning_review", "review"];

interface TaskCardProps {
	task: Task;
	isSelected?: boolean;
	onClick?: () => void;
}

function TaskCard({ task, isSelected, onClick }: TaskCardProps) {
	const isAiPhase = AI_PHASES.includes(task.status);
	const isHumanPhase = HUMAN_PHASES.includes(task.status);
	const isExecuting = useIsTaskExecuting(task.id);

	return (
		<div
			className={cn(
				"group relative rounded-lg border bg-card p-3 cursor-pointer",
				"transition-all duration-150 ease-out",
				"hover:bg-accent/50 hover:border-border/80 hover:shadow-md",
				isSelected && "border-primary bg-primary/5 ring-1 ring-primary shadow-lg shadow-primary/10",
				isExecuting && "border-green-500/50 bg-green-500/5 shadow-lg shadow-green-500/10",
			)}
			onClick={onClick}
			onKeyDown={(e) => e.key === "Enter" && onClick?.()}
			tabIndex={0}
			role="button"
		>
			{/* Executing indicator glow */}
			{isExecuting && (
				<div className="absolute -inset-px rounded-lg bg-gradient-to-r from-green-500/20 via-green-500/10 to-green-500/20 opacity-50 animate-pulse" />
			)}

			<div className="relative">
				<div className="flex items-start justify-between gap-2">
					<div className="flex items-center gap-2 min-w-0">
						{isExecuting && (
							<span className="relative shrink-0">
								<span className="absolute inset-0 w-2.5 h-2.5 rounded-full bg-green-500 animate-ping opacity-75" />
								<span className="relative block w-2.5 h-2.5 rounded-full bg-green-500" />
							</span>
						)}
						<h4 className="text-sm font-medium leading-tight line-clamp-2 text-foreground">
							{task.title}
						</h4>
					</div>
					<div className="flex gap-1 shrink-0">
						{isAiPhase && (
							<Badge variant="secondary" className="text-[10px] px-1.5 py-0 bg-purple-500/10 text-purple-400 border-purple-500/20">
								AI
							</Badge>
						)}
						{isHumanPhase && (
							<Badge variant="outline" className="text-[10px] px-1.5 py-0 bg-amber-500/10 text-amber-400 border-amber-500/20">
								Human
							</Badge>
						)}
					</div>
				</div>

				{task.description && (
					<p className="mt-2 text-xs text-muted-foreground line-clamp-2 leading-relaxed">
						{task.description}
					</p>
				)}

				<div className="mt-3 flex items-center gap-2 text-[10px] text-muted-foreground">
					<span className="font-mono bg-muted/50 px-1.5 py-0.5 rounded">{task.id.slice(0, 8)}</span>
					{task.workspace_path && (
						<span className="truncate opacity-60" title={task.workspace_path}>
							{task.workspace_path.split("/").pop()}
						</span>
					)}
					{isExecuting && (
						<span className="ml-auto text-green-400 font-medium flex items-center gap-1">
							<svg className="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
								<path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" />
							</svg>
							Running
						</span>
					)}
				</div>
			</div>
		</div>
	);
}

export { TaskCard };
