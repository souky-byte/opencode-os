import { useState } from "react";
import { cn } from "@/lib/utils";
import type { SessionActivityMsg } from "@/types/generated/SessionActivityMsg";
import { Icon, getToolIcon } from "@/components/ui/icon";
import { Markdown } from "@/components/ui/markdown";

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

function truncate(str: string, max: number): string {
  return str.length <= max ? str : `${str.slice(0, max)}…`;
}

function getFilename(path: string | undefined): string {
  if (!path) return "";
  return path.split("/").pop() || path;
}

// TodoWrite types
interface TodoItem {
  content: string;
  status: "pending" | "in_progress" | "completed";
  activeForm?: string;
}

function isTodoWrite(toolName: string): boolean {
  return toolName.toLowerCase() === "todowrite";
}

function parseTodoWriteArgs(
  args: Record<string, unknown> | null,
): TodoItem[] | null {
  if (!args) return null;
  const todos = args.todos as TodoItem[] | undefined;
  if (!todos || !Array.isArray(todos)) return null;
  return todos;
}

function getToolSubtitle(
  toolName: string,
  args: Record<string, unknown> | null,
): string | undefined {
  if (!args) return undefined;
  const tool = toolName.toLowerCase();

  if (tool === "read" || tool === "edit" || tool === "write") {
    return getFilename(args.filePath as string | undefined);
  }
  if (tool === "bash" || tool === "shell") {
    return (
      (args.description as string) ||
      truncate((args.command as string) || "", 60)
    );
  }
  if (tool === "glob" || tool === "grep") {
    return (args.pattern as string) || (args.path as string);
  }
  if (tool === "webfetch" || tool === "web_fetch") {
    return truncate((args.url as string) || "", 50);
  }
  if (tool === "task") {
    return (args.description as string) || (args.subagent_type as string);
  }
  if (tool === "todowrite") {
    const todos = parseTodoWriteArgs(args);
    if (todos) {
      const completed = todos.filter((t) => t.status === "completed").length;
      const inProgress = todos.filter((t) => t.status === "in_progress").length;
      return `${completed}/${todos.length} done${inProgress > 0 ? `, ${inProgress} in progress` : ""}`;
    }
  }
  return undefined;
}

// TodoWrite item - special display with todo list
function TodoWriteItem({
  activity,
  isPending,
}: {
  activity: Extract<SessionActivityMsg, { type: "tool_call" | "tool_result" }>;
  isPending: boolean;
}) {
  const [expanded, setExpanded] = useState(false);
  const todos = parseTodoWriteArgs(activity.args);

  if (!todos) return null;

  const completed = todos.filter((t) => t.status === "completed").length;
  const inProgress = todos.filter((t) => t.status === "in_progress").length;
  const pending = todos.filter((t) => t.status === "pending").length;

  return (
    <div className="group my-1">
      <button
        type="button"
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-2 py-1.5 px-2 -mx-2 rounded hover:bg-muted/30 transition-colors text-left"
      >
        {isPending ? (
          <Icon
            name="loading"
            size="xs"
            spin
            className="text-blue-400/70 shrink-0"
          />
        ) : (
          <Icon
            name="check"
            size="xs"
            className="text-emerald-500/70 shrink-0"
          />
        )}
        <Icon name="list" size="xs" className="text-blue-400/60 shrink-0" />
        <span className="text-xs font-medium text-foreground/80">Tasks</span>
        <div className="flex items-center gap-1.5 text-[10px]">
          {completed > 0 && (
            <span className="text-emerald-500/80">{completed} done</span>
          )}
          {inProgress > 0 && (
            <span className="text-blue-400/80">{inProgress} active</span>
          )}
          {pending > 0 && (
            <span className="text-muted-foreground/60">{pending} pending</span>
          )}
        </div>
        <span className="text-[10px] text-muted-foreground/40 ml-auto tabular-nums shrink-0">
          {formatTime(activity.timestamp)}
        </span>
        <Icon
          name={expanded ? "chevron-down" : "chevron-right"}
          size="xs"
          className="text-muted-foreground/40 shrink-0"
        />
      </button>
      {expanded && (
        <div className="ml-4 mt-1 space-y-0.5 border-l-2 border-blue-400/20 pl-3">
          {todos.map((todo, index) => (
            <div
              key={`${todo.content}-${index}`}
              className="flex items-start gap-2 py-0.5"
            >
              {todo.status === "completed" && (
                <Icon
                  name="check-circle"
                  size="xs"
                  className="text-emerald-500/70 mt-0.5 shrink-0"
                />
              )}
              {todo.status === "in_progress" && (
                <Icon
                  name="loading"
                  size="xs"
                  spin
                  className="text-blue-400/70 mt-0.5 shrink-0"
                />
              )}
              {todo.status === "pending" && (
                <Icon
                  name="circle"
                  size="xs"
                  className="text-muted-foreground/40 mt-0.5 shrink-0"
                />
              )}
              <span
                className={cn(
                  "text-xs leading-relaxed",
                  todo.status === "completed" &&
                    "text-muted-foreground/60 line-through",
                  todo.status === "in_progress" && "text-foreground/90",
                  todo.status === "pending" && "text-muted-foreground/70",
                )}
              >
                {todo.content}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// Compact row for tool calls (pending state)
function ToolCallItem({
  activity,
}: {
  activity: Extract<SessionActivityMsg, { type: "tool_call" }>;
}) {
  // Special handling for TodoWrite
  if (isTodoWrite(activity.tool_name)) {
    return <TodoWriteItem activity={activity} isPending={true} />;
  }

  const subtitle = getToolSubtitle(activity.tool_name, activity.args);

  return (
    <div className="group flex items-center gap-2 py-1 px-2 -mx-2 rounded hover:bg-muted/30 transition-colors">
      <Icon
        name="loading"
        size="xs"
        spin
        className="text-muted-foreground/60 shrink-0"
      />
      <span className="text-xs font-medium text-muted-foreground">
        {activity.tool_name}
      </span>
      {subtitle && (
        <span className="text-xs text-muted-foreground/60 truncate">
          {subtitle}
        </span>
      )}
      <span className="text-[10px] text-muted-foreground/40 ml-auto tabular-nums shrink-0">
        {formatTime(activity.timestamp)}
      </span>
    </div>
  );
}

// Compact row for tool results
function ToolResultItem({
  activity,
}: {
  activity: Extract<SessionActivityMsg, { type: "tool_result" }>;
}) {
  const [expanded, setExpanded] = useState(false);

  // Special handling for TodoWrite
  if (isTodoWrite(activity.tool_name)) {
    return <TodoWriteItem activity={activity} isPending={false} />;
  }

  const subtitle = getToolSubtitle(activity.tool_name, activity.args);
  const iconName = getToolIcon(activity.tool_name);
  const hasResult = activity.result && activity.result.length > 0;

  return (
    <div className="group">
      <button
        type="button"
        onClick={() => hasResult && setExpanded(!expanded)}
        disabled={!hasResult}
        className={cn(
          "w-full flex items-center gap-2 py-1 px-2 -mx-2 rounded transition-colors text-left",
          hasResult && "hover:bg-muted/30 cursor-pointer",
          !hasResult && "cursor-default",
        )}
      >
        <Icon
          name={activity.success ? "check" : "error"}
          size="xs"
          className={cn(
            "shrink-0",
            activity.success ? "text-emerald-500/70" : "text-red-500/70",
          )}
        />
        <Icon
          name={iconName}
          size="xs"
          className="text-muted-foreground/50 shrink-0"
        />
        <span className="text-xs font-medium text-foreground/80">
          {activity.tool_name}
        </span>
        {subtitle && (
          <span className="text-xs text-muted-foreground/60 truncate">
            {subtitle}
          </span>
        )}
        <span className="text-[10px] text-muted-foreground/40 ml-auto tabular-nums shrink-0">
          {formatTime(activity.timestamp)}
        </span>
        {hasResult && (
          <Icon
            name={expanded ? "chevron-down" : "chevron-right"}
            size="xs"
            className="text-muted-foreground/40 shrink-0"
          />
        )}
      </button>
      {expanded && hasResult && (
        <pre className="mt-1 ml-4 text-[11px] text-muted-foreground/70 bg-muted/20 rounded px-2 py-1.5 overflow-x-auto max-h-48 overflow-y-auto font-mono whitespace-pre-wrap">
          {activity.result}
        </pre>
      )}
    </div>
  );
}

// Agent message - slightly more prominent but still compact
function AgentMessageItem({
  activity,
}: {
  activity: Extract<SessionActivityMsg, { type: "agent_message" }>;
}) {
  return (
    <div className="py-2 border-l-2 border-primary/30 pl-3 my-1">
      <div className="flex items-center gap-2 mb-1">
        <Icon name="agent" size="xs" className="text-primary/60" />
        <span className="text-xs font-medium text-primary/80">Assistant</span>
        {activity.is_partial && (
          <span className="w-1 h-1 rounded-full bg-amber-400 animate-pulse" />
        )}
        <span className="text-[10px] text-muted-foreground/40 ml-auto tabular-nums">
          {formatTime(activity.timestamp)}
        </span>
      </div>
      <Markdown
        text={activity.content}
        className="text-sm text-foreground/90 leading-relaxed"
      />
    </div>
  );
}

// Reasoning/thinking - subtle and collapsible
function ReasoningItem({
  activity,
}: {
  activity: Extract<SessionActivityMsg, { type: "reasoning" }>;
}) {
  const [expanded, setExpanded] = useState(false);
  const preview = truncate(activity.content.replace(/\*\*/g, "").trim(), 80);

  return (
    <button
      type="button"
      onClick={() => setExpanded(!expanded)}
      className="w-full group flex items-start gap-2 py-1 px-2 -mx-2 rounded hover:bg-muted/30 transition-colors text-left"
    >
      <Icon
        name="reasoning"
        size="xs"
        className="text-violet-400/60 mt-0.5 shrink-0"
      />
      <div className="flex-1 min-w-0">
        {expanded ? (
          <p className="text-xs text-muted-foreground/70 italic whitespace-pre-wrap leading-relaxed">
            {activity.content}
          </p>
        ) : (
          <span className="text-xs text-muted-foreground/60 italic truncate block">
            {preview}
          </span>
        )}
      </div>
      <span className="text-[10px] text-muted-foreground/40 tabular-nums shrink-0">
        {formatTime(activity.timestamp)}
      </span>
    </button>
  );
}

// Finished state - compact but clear
function FinishedItem({
  activity,
}: {
  activity: Extract<SessionActivityMsg, { type: "finished" }>;
}) {
  return (
    <div
      className={cn(
        "flex items-center gap-2 py-2 px-3 -mx-2 rounded mt-2",
        activity.success ? "bg-emerald-500/10" : "bg-red-500/10",
      )}
    >
      <Icon
        name={activity.success ? "check-circle" : "error"}
        size="sm"
        className={activity.success ? "text-emerald-500" : "text-red-500"}
      />
      <span
        className={cn(
          "text-xs font-medium",
          activity.success ? "text-emerald-500" : "text-red-500",
        )}
      >
        {activity.success ? "Completed" : "Failed"}
      </span>
      {activity.error && (
        <span className="text-xs text-red-400/80 truncate flex-1">
          {activity.error}
        </span>
      )}
      <span className="text-[10px] text-muted-foreground/40 tabular-nums ml-auto">
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
    case "finished":
      return <FinishedItem activity={activity} />;
    case "step_start":
    case "json_patch":
      return null;
    default:
      return null;
  }
}
