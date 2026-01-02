import {
	AlertTriangle,
	BookOpen,
	Check,
	CheckCircle,
	ChevronDown,
	ChevronRight,
	Code,
	FileCode,
	FileSearch,
	FileText,
	FolderOpen,
	GitBranch,
	Globe,
	Info,
	Loader2,
	type LucideIcon,
	MessageSquare,
	Minus,
	Pencil,
	Search,
	Terminal,
	ListTodo,
	Brain,
	Sparkles,
	AlertCircle,
	XCircle,
	Wrench,
	Play,
	Clock,
} from "lucide-react";
import { cn } from "@/lib/utils";

export type IconName =
	| "read"
	| "edit"
	| "write"
	| "bash"
	| "terminal"
	| "glob"
	| "grep"
	| "search"
	| "webfetch"
	| "task"
	| "todo"
	| "text"
	| "reasoning"
	| "agent"
	| "file"
	| "folder"
	| "code"
	| "git"
	| "check"
	| "check-circle"
	| "error"
	| "warning"
	| "info"
	| "alert-circle"
	| "alert-triangle"
	| "loading"
	| "chevron-down"
	| "chevron-right"
	| "tool"
	| "play"
	| "clock"
	| "minus";

const ICON_MAP: Record<IconName, LucideIcon> = {
	read: BookOpen,
	edit: Pencil,
	write: FileCode,
	bash: Terminal,
	terminal: Terminal,
	glob: FileSearch,
	grep: Search,
	search: Search,
	webfetch: Globe,
	task: Sparkles,
	todo: ListTodo,
	text: MessageSquare,
	reasoning: Brain,
	agent: Sparkles,
	file: FileText,
	folder: FolderOpen,
	code: Code,
	git: GitBranch,
	check: Check,
	"check-circle": CheckCircle,
	error: XCircle,
	warning: AlertCircle,
	info: Info,
	"alert-circle": AlertCircle,
	"alert-triangle": AlertTriangle,
	loading: Loader2,
	"chevron-down": ChevronDown,
	"chevron-right": ChevronRight,
	tool: Wrench,
	play: Play,
	clock: Clock,
	minus: Minus,
};

interface IconProps {
	name: IconName;
	size?: "xs" | "sm" | "md" | "lg";
	className?: string;
	spin?: boolean;
}

const SIZE_MAP = {
	xs: "h-3 w-3",
	sm: "h-4 w-4",
	md: "h-5 w-5",
	lg: "h-6 w-6",
};

export function Icon({ name, size = "sm", className, spin }: IconProps) {
	const IconComponent = ICON_MAP[name];

	if (!IconComponent) {
		return null;
	}

	return (
		<IconComponent
			className={cn(SIZE_MAP[size], spin && "animate-spin", className)}
		/>
	);
}

export function getToolIcon(toolName: string): IconName {
	switch (toolName.toLowerCase()) {
		case "read":
			return "read";
		case "edit":
			return "edit";
		case "write":
			return "write";
		case "bash":
		case "shell":
			return "bash";
		case "glob":
			return "glob";
		case "grep":
			return "grep";
		case "webfetch":
		case "web_fetch":
			return "webfetch";
		case "task":
		case "agent":
			return "task";
		case "todowrite":
		case "todoread":
		case "todo":
			return "todo";
		default:
			return "tool";
	}
}

export function getToolStatus(toolName: string): string {
	switch (toolName.toLowerCase()) {
		case "task":
			return "Delegating work";
		case "todowrite":
		case "todoread":
			return "Planning next steps";
		case "read":
			return "Gathering context";
		case "list":
		case "grep":
		case "glob":
			return "Searching the codebase";
		case "webfetch":
		case "web_fetch":
			return "Searching the web";
		case "edit":
		case "write":
			return "Making edits";
		case "bash":
		case "shell":
			return "Running commands";
		default:
			return "Processing";
	}
}
