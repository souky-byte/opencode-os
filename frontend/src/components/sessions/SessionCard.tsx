import { useState } from "react";
import type { Session, SessionPhase, SessionStatus } from "@/api/generated/model";
import { ActivityFeed } from "@/components/activity/ActivityFeed";
import { Badge } from "@/components/ui/badge";
import { useSessionActivitySSE } from "@/hooks/useSessionActivitySSE";
import { cn } from "@/lib/utils";

const PHASE_COLORS: Record<SessionPhase, string> = {
	planning: "bg-blue-500/10 text-blue-500 border-blue-500/20",
	implementation: "bg-purple-500/10 text-purple-500 border-purple-500/20",
	fix: "bg-red-500/10 text-red-500 border-red-500/20",
	review: "bg-amber-500/10 text-amber-500 border-amber-500/20",
};

const STATUS_COLORS: Record<SessionStatus, string> = {
	pending: "bg-gray-500/10 text-gray-500",
	running: "bg-green-500/10 text-green-500",
	completed: "bg-blue-500/10 text-blue-500",
	failed: "bg-red-500/10 text-red-500",
	aborted: "bg-orange-500/10 text-orange-500",
};

interface SessionCardProps {
	session: Session;
}

export function SessionCard({ session }: SessionCardProps) {
	const [expanded, setExpanded] = useState(false);

	const { activities, isConnected, isFinished, error } = useSessionActivitySSE(session.id, {
		enabled: expanded,
	});

	const formatDate = (dateStr: string | undefined | null) => {
		if (!dateStr) {
			return "-";
		}
		return new Date(dateStr).toLocaleString([], {
			month: "short",
			day: "numeric",
			hour: "2-digit",
			minute: "2-digit",
		});
	};

	const isRunning = session.status === "running";

	return (
		<div className="rounded-lg border bg-card">
			<button
				type="button"
				onClick={() => setExpanded(!expanded)}
				className="w-full p-4 text-left hover:bg-accent/50 transition-colors"
			>
				<div className="flex items-center justify-between gap-4">
					<div className="flex items-center gap-3 min-w-0">
						<Badge className={cn("capitalize", PHASE_COLORS[session.phase])}>{session.phase}</Badge>
						<Badge className={cn("capitalize", STATUS_COLORS[session.status])}>
							{isRunning && (
								<span className="mr-1.5 inline-block w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse" />
							)}
							{session.status}
						</Badge>
					</div>

					<div className="flex items-center gap-4 text-xs text-muted-foreground shrink-0">
						<span>Started: {formatDate(session.started_at)}</span>
						{session.completed_at && <span>Completed: {formatDate(session.completed_at)}</span>}
						<span className={cn("transition-transform", expanded ? "rotate-180" : "")}>▼</span>
					</div>
				</div>

				<div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
					<span className="font-mono">Task: {session.task_id.slice(0, 8)}</span>
					{session.opencode_session_id && (
						<span className="font-mono">OpenCode: {session.opencode_session_id}</span>
					)}
				</div>
			</button>

			{expanded && (
				<div className="border-t h-80">
					<ActivityFeed
						activities={activities}
						isConnected={isConnected}
						isFinished={isFinished}
						error={error}
					/>
				</div>
			)}
		</div>
	);
}
