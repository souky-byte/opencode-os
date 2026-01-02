import { useState } from "react";
import { cn } from "@/lib/utils";
import type { SessionActivityMsg } from "@/types/generated/SessionActivityMsg";
import { Icon, getToolIcon, getToolStatus } from "@/components/ui/icon";
import { Markdown } from "@/components/ui/markdown";
import { Collapsible } from "@/components/ui/collapsible";

interface ActivityItemProps {
	activity: SessionActivityMsg;
}

function formatTime(timestamp: string): string {
	return new Date(timestamp).toLocaleTimeString([], {
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
	});
}

function truncateString(str: string, maxLen: number): string {
	if (str.length <= maxLen) {
		return str;
	}
	return `${str.slice(0, maxLen)}...`;
}

function getFilename(path: string | undefined): string {
	if (!path) return "";
	const parts = path.split("/");
	return parts[parts.length - 1] || path;
}


function formatToolArgs(
	args: Record<string, unknown> | null,
): { summary: string; full: string } | null {
	if (!args || Object.keys(args).length === 0) return null;

	const entries = Object.entries(args);
	const summary = entries
		.slice(0, 2)
		.map(([k, v]) => {
			const val = typeof v === "string" ? v : JSON.stringify(v);
			return `${k}: ${truncateString(val, 30)}`;
		})
		.join(", ");

	return {
		summary:
			entries.length > 2 ? `${summary} (+${entries.length - 2} more)` : summary,
		full: JSON.stringify(args, null, 2),
	};
}

function getToolSubtitle(
	toolName: string,
	args: Record<string, unknown> | null,
): string | undefined {
	if (!args) return undefined;

	const tool = toolName.toLowerCase();

	if (tool === "read" || tool === "edit" || tool === "write") {
		const path = args.filePath as string | undefined;
		return path ? getFilename(path) : undefined;
	}

	if (tool === "bash" || tool === "shell") {
		return (
			(args.description as string) ||
			truncateString((args.command as string) || "", 50)
		);
	}

	if (tool === "glob" || tool === "grep") {
		return (args.pattern as string) || (args.path as string);
	}

	if (tool === "webfetch" || tool === "web_fetch") {
		const url = args.url as string;
		return url ? truncateString(url, 40) : undefined;
	}

	if (tool === "task") {
		return (args.description as string) || (args.subagent_type as string);
	}

	return undefined;
}

function ToolCallItem({
	activity,
}: {
	activity: Extract<SessionActivityMsg, { type: "tool_call" }>;
}) {
	const [isExpanded, setIsExpanded] = useState(false);
	const argsFormatted = formatToolArgs(activity.args);
	const subtitle = getToolSubtitle(activity.tool_name, activity.args);
	const statusText = getToolStatus(activity.tool_name);

	return (
		<Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
			<div className="rounded-lg bg-primary/5 border border-primary/20">
				<Collapsible.Trigger>
					<div className="flex items-center gap-3 p-3">
						<div className="shrink-0 p-1.5 rounded-md bg-primary/10">
							<Icon name="loading" size="sm" spin className="text-primary" />
						</div>

						<div className="flex-1 min-w-0 flex items-center gap-2">
							<span className="text-sm font-medium text-foreground">
								{activity.tool_name}
							</span>
							{subtitle && (
								<span className="text-sm text-muted-foreground truncate">
									{subtitle}
								</span>
							)}
						</div>

						<div className="flex items-center gap-2 shrink-0">
							<span className="text-xs text-primary/70">{statusText}</span>
							<span className="text-xs text-muted-foreground">
								{formatTime(activity.timestamp)}
							</span>
							{argsFormatted && <Collapsible.Arrow />}
						</div>
					</div>
				</Collapsible.Trigger>

				{argsFormatted && (
					<Collapsible.Content>
						<div className="px-3 pb-3 pt-0">
							<pre className="text-xs text-muted-foreground overflow-x-auto p-3 bg-muted/30 rounded-md font-mono">
								{argsFormatted.full}
							</pre>
						</div>
					</Collapsible.Content>
				)}
			</div>
		</Collapsible>
	);
}

function ToolResultItem({
	activity,
}: {
	activity: Extract<SessionActivityMsg, { type: "tool_result" }>;
}) {
	const [isExpanded, setIsExpanded] = useState(false);
	const hasResult = activity.result && activity.result.length > 0;
	const hasLongResult = activity.result && activity.result.length > 150;
	const subtitle = getToolSubtitle(activity.tool_name, activity.args);
	const iconName = getToolIcon(activity.tool_name);

	return (
		<Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
			<div
				className={cn(
					"rounded-lg border",
					activity.success
						? "bg-emerald-500/5 border-emerald-500/20"
						: "bg-red-500/5 border-red-500/20",
				)}
			>
				<Collapsible.Trigger>
					<div className="flex items-center gap-3 p-3">
						<div
							className={cn(
								"shrink-0 p-1.5 rounded-md",
								activity.success ? "bg-emerald-500/10" : "bg-red-500/10",
							)}
						>
							<Icon
								name={activity.success ? "check" : "error"}
								size="sm"
								className={
									activity.success ? "text-emerald-400" : "text-red-400"
								}
							/>
						</div>

						<div className="flex-1 min-w-0 flex items-center gap-2">
							<Icon
								name={iconName}
								size="xs"
								className="text-muted-foreground shrink-0"
							/>
							<span className="text-sm font-medium text-foreground">
								{activity.tool_name}
							</span>
							{subtitle && (
								<span className="text-sm text-muted-foreground truncate">
									{subtitle}
								</span>
							)}
						</div>

						<div className="flex items-center gap-2 shrink-0">
							<span
								className={cn(
									"text-[10px] px-2 py-0.5 rounded-full font-medium",
									activity.success
										? "bg-emerald-500/20 text-emerald-400"
										: "bg-red-500/20 text-red-400",
								)}
							>
								{activity.success ? "done" : "failed"}
							</span>
							<span className="text-xs text-muted-foreground">
								{formatTime(activity.timestamp)}
							</span>
							{hasResult && <Collapsible.Arrow />}
						</div>
					</div>
				</Collapsible.Trigger>

				{hasResult && (
					<Collapsible.Content>
						<div className="px-3 pb-3 pt-0">
							<pre
								className={cn(
									"text-xs text-muted-foreground overflow-x-auto p-3 bg-muted/30 rounded-md font-mono whitespace-pre-wrap",
									hasLongResult && "max-h-64 overflow-y-auto",
								)}
							>
								{activity.result}
							</pre>
						</div>
					</Collapsible.Content>
				)}
			</div>
		</Collapsible>
	);
}

function AgentMessageItem({
	activity,
}: {
	activity: Extract<SessionActivityMsg, { type: "agent_message" }>;
}) {
	return (
		<div className="rounded-lg bg-card border border-border/50 p-4">
			<div className="flex items-start gap-3">
				<div className="shrink-0 mt-0.5 p-1.5 rounded-md bg-primary/10">
					<Icon name="agent" size="sm" className="text-primary" />
				</div>
				<div className="flex-1 min-w-0">
					<div className="flex items-center gap-2 mb-2">
						<span className="text-sm font-medium text-foreground">
							Assistant
						</span>
						{activity.is_partial && (
							<span className="text-[10px] px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-400 font-medium flex items-center gap-1">
								<span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-pulse" />
								typing
							</span>
						)}
						<span className="text-xs text-muted-foreground ml-auto">
							{formatTime(activity.timestamp)}
						</span>
					</div>
					<Markdown
						text={activity.content}
						className="text-sm text-foreground/90"
					/>
				</div>
			</div>
		</div>
	);
}

function ReasoningItem({
	activity,
}: {
	activity: Extract<SessionActivityMsg, { type: "reasoning" }>;
}) {
	const [isExpanded, setIsExpanded] = useState(false);
	const isLong = activity.content.length > 200;

	// Extract first bold section as title if present
	const match = activity.content.trimStart().match(/^\*\*(.+?)\*\*/);
	const title = match ? match[1].trim() : null;

	return (
		<Collapsible open={isExpanded} onOpenChange={setIsExpanded}>
			<div className="rounded-lg bg-purple-500/5 border border-purple-500/20">
				<Collapsible.Trigger>
					<div className="flex items-center gap-3 p-3">
						<div className="shrink-0 p-1.5 rounded-md bg-purple-500/10">
							<Icon name="reasoning" size="sm" className="text-purple-400" />
						</div>

						<div className="flex-1 min-w-0">
							<span className="text-sm font-medium text-purple-300">
								Thinking
								{title && (
									<span className="text-muted-foreground font-normal">
										{" "}
										· {title}
									</span>
								)}
							</span>
						</div>

						<div className="flex items-center gap-2 shrink-0">
							<span className="text-xs text-muted-foreground">
								{formatTime(activity.timestamp)}
							</span>
							{isLong && <Collapsible.Arrow />}
						</div>
					</div>
				</Collapsible.Trigger>

				<Collapsible.Content>
					<div className="px-3 pb-3 pt-0">
						<p className="text-sm text-muted-foreground/80 italic whitespace-pre-wrap leading-relaxed">
							{activity.content}
						</p>
					</div>
				</Collapsible.Content>

				{!isLong && !isExpanded && (
					<div className="px-3 pb-3 pt-0">
						<p className="text-sm text-muted-foreground/80 italic whitespace-pre-wrap leading-relaxed">
							{activity.content}
						</p>
					</div>
				)}
			</div>
		</Collapsible>
	);
}

function StepStartItem({
	activity,
}: {
	activity: Extract<SessionActivityMsg, { type: "step_start" }>;
}) {
	return (
		<div className="flex items-center gap-3 py-2">
			<div className="flex-1 h-px bg-gradient-to-r from-transparent via-border to-border" />
			<div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-muted/50 border border-border/50">
				<Icon name="play" size="xs" className="text-muted-foreground" />
				<span className="text-xs font-medium text-muted-foreground">
					{activity.step_name ?? "Step"}
				</span>
			</div>
			<span className="text-[11px] text-muted-foreground">
				{formatTime(activity.timestamp)}
			</span>
			<div className="flex-1 h-px bg-gradient-to-l from-transparent via-border to-border" />
		</div>
	);
}

function FinishedItem({
	activity,
}: {
	activity: Extract<SessionActivityMsg, { type: "finished" }>;
}) {
	return (
		<div
			className={cn(
				"flex items-center gap-4 p-4 rounded-lg border",
				activity.success
					? "bg-emerald-500/5 border-emerald-500/20"
					: "bg-red-500/5 border-red-500/20",
			)}
		>
			<div
				className={cn(
					"w-10 h-10 rounded-lg flex items-center justify-center",
					activity.success ? "bg-emerald-500/15" : "bg-red-500/15",
				)}
			>
				<Icon
					name={activity.success ? "check-circle" : "error"}
					size="lg"
					className={activity.success ? "text-emerald-400" : "text-red-400"}
				/>
			</div>
			<div className="flex-1">
				<span
					className={cn(
						"text-sm font-semibold",
						activity.success ? "text-emerald-300" : "text-red-300",
					)}
				>
					Session {activity.success ? "completed" : "failed"}
				</span>
				{activity.error && (
					<p className="mt-1 text-sm text-red-400">{activity.error}</p>
				)}
			</div>
			<span className="text-xs text-muted-foreground">
				{formatTime(activity.timestamp)}
			</span>
		</div>
	);
}

export function ActivityItem({ activity }: ActivityItemProps) {
	switch (activity.type) {
		case "tool_call":
			return <ToolCallItem activity={activity} />;
		case "tool_result":
			return <ToolResultItem activity={activity} />;
		case "agent_message":
			return <AgentMessageItem activity={activity} />;
		case "reasoning":
			return <ReasoningItem activity={activity} />;
		case "step_start":
			return <StepStartItem activity={activity} />;
		case "finished":
			return <FinishedItem activity={activity} />;
		case "json_patch":
			return null;
		default:
			return null;
	}
}
