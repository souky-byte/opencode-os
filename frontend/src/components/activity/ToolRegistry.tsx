import type { ReactNode } from "react";
import { BasicTool } from "./BasicTool";
import { cn } from "@/lib/utils";

export interface ToolRenderProps {
	tool: string;
	input: Record<string, unknown>;
	output?: string;
	error?: string;
	status: "pending" | "running" | "completed" | "error";
	metadata?: Record<string, unknown>;
}

type ToolRenderer = (props: ToolRenderProps) => ReactNode;

const toolRenderers: Record<string, ToolRenderer> = {};

export function registerTool(name: string, renderer: ToolRenderer) {
	toolRenderers[name] = renderer;
}

export function getToolRenderer(name: string): ToolRenderer | undefined {
	return toolRenderers[name];
}

function getFilename(path: string | undefined): string {
	if (!path) return "";
	const parts = path.split("/");
	return parts[parts.length - 1] || path;
}

function getDirectory(path: string | undefined): string {
	if (!path) return "";
	const parts = path.split("/");
	parts.pop();
	return parts.join("/");
}

// Tool output wrapper
function ToolOutput({
	children,
	scrollable = true,
	className,
}: { children: ReactNode; scrollable?: boolean; className?: string }) {
	return (
		<div
			className={cn(
				"text-sm font-mono text-muted-foreground",
				"bg-muted/30 rounded-md p-3",
				scrollable && "max-h-64 overflow-auto",
				className,
			)}
		>
			<pre className="whitespace-pre-wrap break-words">{children}</pre>
		</div>
	);
}

// Read tool
registerTool("read", (props) => {
	const filePath = props.input.filePath as string | undefined;
	const args: string[] = [];
	if (props.input.offset) args.push(`offset=${props.input.offset}`);
	if (props.input.limit) args.push(`limit=${props.input.limit}`);

	return (
		<BasicTool
			icon="read"
			status={props.status}
			trigger={{
				title: "Read",
				subtitle: filePath ? getFilename(filePath) : undefined,
				args: args.length > 0 ? args : undefined,
			}}
		/>
	);
});

// Edit tool
registerTool("edit", (props) => {
	const filePath = props.input.filePath as string | undefined;
	const directory = getDirectory(filePath);
	const filename = getFilename(filePath);

	return (
		<BasicTool
			icon="edit"
			status={props.status}
			trigger={
				<div className="flex items-center gap-2 min-w-0">
					<span className="text-sm font-medium text-foreground shrink-0">
						Edit
					</span>
					{directory && (
						<span className="text-sm text-muted-foreground truncate">
							{directory}/
						</span>
					)}
					<span className="text-sm text-foreground">{filename}</span>
				</div>
			}
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
});

// Write tool
registerTool("write", (props) => {
	const filePath = props.input.filePath as string | undefined;
	const directory = getDirectory(filePath);
	const filename = getFilename(filePath);

	return (
		<BasicTool
			icon="write"
			status={props.status}
			trigger={
				<div className="flex items-center gap-2 min-w-0">
					<span className="text-sm font-medium text-foreground shrink-0">
						Write
					</span>
					{directory && (
						<span className="text-sm text-muted-foreground truncate">
							{directory}/
						</span>
					)}
					<span className="text-sm text-foreground">{filename}</span>
				</div>
			}
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
});

// Bash/Shell tool
registerTool("bash", (props) => {
	const command = (props.input.command as string) || "";
	const description = props.input.description as string | undefined;

	return (
		<BasicTool
			icon="bash"
			status={props.status}
			trigger={{
				title: "Shell",
				subtitle: description || command.slice(0, 50),
			}}
			defaultOpen={props.status === "running"}
		>
			<ToolOutput>
				<span className="text-muted-foreground">$ </span>
				{command}
				{props.output && (
					<>
						{"\n\n"}
						{props.output}
					</>
				)}
			</ToolOutput>
		</BasicTool>
	);
});

// Glob tool
registerTool("glob", (props) => {
	const pattern = props.input.pattern as string | undefined;
	const path = props.input.path as string | undefined;

	return (
		<BasicTool
			icon="glob"
			status={props.status}
			trigger={{
				title: "Glob",
				subtitle: path || "/",
				args: pattern ? [`pattern=${pattern}`] : undefined,
			}}
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
});

// Grep tool
registerTool("grep", (props) => {
	const pattern = props.input.pattern as string | undefined;
	const path = props.input.path as string | undefined;

	return (
		<BasicTool
			icon="grep"
			status={props.status}
			trigger={{
				title: "Grep",
				subtitle: path || "/",
				args: pattern ? [`pattern=${pattern}`] : undefined,
			}}
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
});

// WebFetch tool
registerTool("webfetch", (props) => {
	const url = props.input.url as string | undefined;

	return (
		<BasicTool
			icon="webfetch"
			status={props.status}
			trigger={{
				title: "WebFetch",
				subtitle: url,
			}}
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
});

// Task/Agent tool
registerTool("task", (props) => {
	const description = props.input.description as string | undefined;
	const agentType = props.input.subagent_type as string | undefined;

	return (
		<BasicTool
			icon="task"
			status={props.status}
			trigger={{
				title: `${agentType || "Task"} Agent`,
				subtitle: description,
			}}
			defaultOpen
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
});

// TodoWrite tool
registerTool("todowrite", (props) => {
	const todos = props.input.todos as
		| Array<{ content: string; status: string }>
		| undefined;
	const completed = todos?.filter((t) => t.status === "completed").length || 0;
	const total = todos?.length || 0;

	return (
		<BasicTool
			icon="todo"
			status={props.status}
			trigger={{
				title: "To-dos",
				subtitle: total > 0 ? `${completed}/${total}` : undefined,
			}}
			defaultOpen
		>
			{todos && todos.length > 0 && (
				<div className="space-y-1">
					{todos.map((todo, i) => (
						<div key={`todo-${i}`} className="flex items-start gap-2 text-sm">
							<span
								className={cn(
									"shrink-0 mt-0.5",
									todo.status === "completed"
										? "text-green-400"
										: "text-muted-foreground",
								)}
							>
								{todo.status === "completed" ? "✓" : "○"}
							</span>
							<span
								className={cn(
									todo.status === "completed" &&
										"text-muted-foreground line-through",
								)}
							>
								{todo.content}
							</span>
						</div>
					))}
				</div>
			)}
		</BasicTool>
	);
});

// Default/fallback renderer
export function renderTool(props: ToolRenderProps): ReactNode {
	const renderer = getToolRenderer(props.tool);

	if (renderer) {
		return renderer(props);
	}

	// Fallback for unknown tools
	return (
		<BasicTool
			icon="tool"
			status={props.status}
			trigger={{ title: props.tool }}
		>
			{props.output && <ToolOutput>{props.output}</ToolOutput>}
		</BasicTool>
	);
}

// Export all for convenience
export { BasicTool, ToolOutput };
