import { useMemo, useState } from "react";
import type { SessionPhase, SessionStatus } from "@/api/generated/model";
import { useListSessions } from "@/api/generated/sessions/sessions";
import { Button } from "@/components/ui/button";
import { Loader } from "@/components/ui/loader";
import { cn } from "@/lib/utils";
import { SessionCard } from "./SessionCard";

type PhaseFilter = SessionPhase | "all";
type StatusFilter = SessionStatus | "all";

const PHASE_OPTIONS: { value: PhaseFilter; label: string }[] = [
	{ value: "all", label: "All Phases" },
	{ value: "planning", label: "Planning" },
	{ value: "implementation", label: "Implementation" },
	{ value: "review", label: "Review" },
];

const STATUS_OPTIONS: { value: StatusFilter; label: string }[] = [
	{ value: "all", label: "All Statuses" },
	{ value: "running", label: "Running" },
	{ value: "pending", label: "Pending" },
	{ value: "completed", label: "Completed" },
	{ value: "failed", label: "Failed" },
	{ value: "aborted", label: "Aborted" },
];

export function SessionsList() {
	const [phaseFilter, setPhaseFilter] = useState<PhaseFilter>("all");
	const [statusFilter, setStatusFilter] = useState<StatusFilter>("all");

	// Sessions are updated in real-time via SSE events - no polling needed
	const {
		data: sessionsResponse,
		isLoading,
		error,
	} = useListSessions({
		query: { staleTime: 30000 },
	});

	const sessions = sessionsResponse?.data ?? [];

	const filteredSessions = useMemo(
		() =>
			sessions
				.filter((s) => phaseFilter === "all" || s.phase === phaseFilter)
				.filter((s) => statusFilter === "all" || s.status === statusFilter)
				.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()),
		[sessions, phaseFilter, statusFilter],
	);

	const runningCount = sessions.filter((s) => s.status === "running").length;

	if (isLoading) {
		return (
			<div className="flex items-center justify-center py-12">
				<Loader message="Loading sessions..." />
			</div>
		);
	}

	if (error) {
		return (
			<div className="rounded-lg border border-destructive/50 bg-destructive/10 p-4">
				<p className="text-sm text-destructive">Failed to load sessions. Please try again.</p>
			</div>
		);
	}

	return (
		<div className="space-y-4">
			<div className="flex flex-wrap items-center gap-4">
				<div className="flex items-center gap-2">
					<span className="text-sm text-muted-foreground">Phase:</span>
					<div className="flex gap-1">
						{PHASE_OPTIONS.map((option) => (
							<Button
								key={option.value}
								size="sm"
								variant={phaseFilter === option.value ? "default" : "outline"}
								onClick={() => setPhaseFilter(option.value)}
								className="h-7 text-xs"
							>
								{option.label}
							</Button>
						))}
					</div>
				</div>

				<div className="flex items-center gap-2">
					<span className="text-sm text-muted-foreground">Status:</span>
					<div className="flex gap-1">
						{STATUS_OPTIONS.map((option) => (
							<Button
								key={option.value}
								size="sm"
								variant={statusFilter === option.value ? "default" : "outline"}
								onClick={() => setStatusFilter(option.value)}
								className={cn(
									"h-7 text-xs",
									option.value === "running" && runningCount > 0 && "relative",
								)}
							>
								{option.label}
								{option.value === "running" && runningCount > 0 && (
									<span className="ml-1 inline-flex items-center justify-center px-1.5 min-w-[1.25rem] h-4 text-[10px] font-medium rounded-full bg-green-500 text-white">
										{runningCount}
									</span>
								)}
							</Button>
						))}
					</div>
				</div>
			</div>

			<div className="text-sm text-muted-foreground">
				{filteredSessions.length} session{filteredSessions.length !== 1 ? "s" : ""}
				{phaseFilter !== "all" || statusFilter !== "all" ? " (filtered)" : ""}
			</div>

			{filteredSessions.length === 0 ? (
				<div className="rounded-lg border bg-muted/30 p-8 text-center">
					<p className="text-muted-foreground">
						{sessions.length === 0
							? "No sessions yet. Execute a task phase to create one."
							: "No sessions match the current filters."}
					</p>
				</div>
			) : (
				<div className="space-y-3">
					{filteredSessions.map((session) => (
						<SessionCard key={session.id} session={session} />
					))}
				</div>
			)}
		</div>
	);
}
