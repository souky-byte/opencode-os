import { useState } from "react";
import type { PhaseInfo, PhaseStatus } from "@/api/generated/model";
import { useGetTaskPhases } from "@/api/generated/phases/phases";
import { ActivityFeed } from "@/components/activity/ActivityFeed";
import { Badge } from "@/components/ui/badge";
import { Loader } from "@/components/ui/loader";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useSessionActivitySSE } from "@/hooks/useSessionActivitySSE";
import { cn } from "@/lib/utils";
import {
	CheckCircle2,
	Circle,
	Loader2,
	ChevronDown,
	ChevronRight,
} from "lucide-react";

interface PhasesListProps {
	taskId: string;
	className?: string;
}

const PHASE_STATUS_CONFIG: Record<PhaseStatus, {
	icon: React.ElementType;
	label: string;
	color: string;
	bgColor: string;
}> = {
	pending: {
		icon: Circle,
		label: "Pending",
		color: "text-muted-foreground",
		bgColor: "bg-muted",
	},
	running: {
		icon: Loader2,
		label: "Running",
		color: "text-blue-500",
		bgColor: "bg-blue-500/10",
	},
	completed: {
		icon: CheckCircle2,
		label: "Completed",
		color: "text-green-500",
		bgColor: "bg-green-500/10",
	},
};

function PhaseItem({
	phase,
	isFirst,
	isLast,
	isExpanded,
	onToggle,
}: {
	phase: PhaseInfo;
	isFirst: boolean;
	isLast: boolean;
	isExpanded: boolean;
	onToggle: () => void;
}) {
	const config = PHASE_STATUS_CONFIG[phase.status];
	const Icon = config.icon;
	const hasSession = !!phase.session_id;

	// Subscribe to activity stream for this phase's session
	const {
		activities,
		isConnected,
		isFinished,
		error: activityError,
	} = useSessionActivitySSE(isExpanded && hasSession ? (phase.session_id ?? null) : null, {
		enabled: isExpanded && hasSession,
	});

	return (
		<div className="relative">
			{/* Vertical line connecting phases */}
			{!isFirst && (
				<div
					className={cn(
						"absolute left-3 -top-3 w-0.5 h-3",
						phase.status === "completed" ? "bg-green-500/50" : "bg-border"
					)}
				/>
			)}
			{!isLast && (
				<div
					className={cn(
						"absolute left-3 top-6 w-0.5 bottom-0",
						phase.status === "completed" ? "bg-green-500/50" : "bg-border"
					)}
				/>
			)}

			<div className="relative">
				{/* Phase header - clickable */}
				<button
					type="button"
					onClick={onToggle}
					className={cn(
						"w-full flex items-start gap-3 p-2 rounded-lg text-left transition-colors",
						"hover:bg-accent/50",
						isExpanded && "bg-accent/30"
					)}
				>
					{/* Status icon */}
					<div className={cn(
						"shrink-0 w-6 h-6 rounded-full flex items-center justify-center",
						config.bgColor
					)}>
						<Icon
							className={cn(
								"w-4 h-4",
								config.color,
								phase.status === "running" && "animate-spin"
							)}
						/>
					</div>

					{/* Phase info */}
					<div className="flex-1 min-w-0">
						<div className="flex items-center gap-2">
							<span className="font-medium truncate">
								Phase {phase.number}: {phase.title}
							</span>
							<Badge
								variant="outline"
								className={cn("text-xs shrink-0", config.color)}
							>
								{config.label}
							</Badge>
						</div>

						{/* Summary for completed phases */}
						{phase.summary && (
							<p className="mt-1 text-sm text-muted-foreground line-clamp-2">
								{phase.summary.summary}
							</p>
						)}

						{/* Changed files for completed phases */}
						{phase.summary?.files_changed && phase.summary.files_changed.length > 0 && (
							<div className="mt-1 flex flex-wrap gap-1">
								{phase.summary.files_changed.slice(0, 3).map((file: string) => (
									<span
										key={file}
										className="text-xs bg-muted px-1.5 py-0.5 rounded font-mono"
									>
										{file.split('/').pop()}
									</span>
								))}
								{phase.summary.files_changed.length > 3 && (
									<span className="text-xs text-muted-foreground">
										+{phase.summary.files_changed.length - 3} more
									</span>
								)}
							</div>
						)}
					</div>

					{/* Expand/collapse indicator */}
					{hasSession && (
						<div className="shrink-0 text-muted-foreground">
							{isExpanded ? (
								<ChevronDown className="w-4 h-4" />
							) : (
								<ChevronRight className="w-4 h-4" />
							)}
						</div>
					)}
				</button>

				{/* Expanded activity feed */}
				{isExpanded && hasSession && (
					<div className="mt-2 ml-9 border rounded-lg overflow-hidden">
						<div className="bg-muted/30 px-3 py-1.5 text-xs text-muted-foreground border-b">
							Session: <span className="font-mono">{phase.session_id?.slice(0, 8)}...</span>
							{isConnected && phase.status === "running" && (
								<span className="ml-2 inline-flex items-center gap-1">
									<span className="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
									Live
								</span>
							)}
						</div>
						<ScrollArea className="h-[300px]">
							<ActivityFeed
								activities={activities}
								isConnected={isConnected}
								isFinished={isFinished}
								error={activityError}
								className="p-2"
							/>
						</ScrollArea>
					</div>
				)}
			</div>
		</div>
	);
}

export function PhasesList({ taskId, className }: PhasesListProps) {
	const [expandedPhase, setExpandedPhase] = useState<number | null>(null);

	const { data: phasesData, isLoading, error } = useGetTaskPhases(taskId, {
		query: {
			staleTime: 10000,
			refetchInterval: (query) => {
				// Refetch more frequently when there's a running phase
				const data = query.state.data;
				if (data?.status === 200 && data.data.current_phase !== null) {
					return 5000;
				}
				return 30000;
			},
		},
	});

	if (isLoading) {
		return (
			<div className={cn("flex items-center justify-center p-8", className)}>
				<Loader size="sm" message="Loading phases..." />
			</div>
		);
	}

	if (error) {
		return (
			<div className={cn("p-4 text-sm text-destructive", className)}>
				Failed to load phases
			</div>
		);
	}

	const phases = phasesData?.status === 200 ? phasesData.data : null;

	if (!phases || phases.phases.length === 0) {
		return (
			<div className={cn("p-4 text-sm text-muted-foreground", className)}>
				No implementation phases detected
			</div>
		);
	}

	// Auto-expand running phase if none selected
	const effectiveExpanded = expandedPhase ?? (phases.current_phase ?? null);

	return (
		<div className={cn("space-y-1", className)}>
			{/* Header */}
			<div className="flex items-center justify-between mb-3">
				<h4 className="text-sm font-medium text-muted-foreground">
					Implementation Phases
				</h4>
				{phases.current_phase && (
					<Badge variant="outline" className="text-xs">
						{phases.current_phase} / {phases.total_phases}
					</Badge>
				)}
			</div>

			{/* Phases list */}
			<div className="space-y-1">
				{phases.phases.map((phase: PhaseInfo, index: number) => (
					<PhaseItem
						key={phase.number}
						phase={phase}
						isFirst={index === 0}
						isLast={index === phases.phases.length - 1}
						isExpanded={effectiveExpanded === phase.number}
						onToggle={() => {
							setExpandedPhase(
								effectiveExpanded === phase.number ? null : phase.number
							);
						}}
					/>
				))}
			</div>
		</div>
	);
}
