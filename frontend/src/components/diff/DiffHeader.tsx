import type { DiffFile } from "diff2html/lib/types";
import { ChevronDown, ChevronRight } from "lucide-react";
import { getFileDisplayName, getFileLanguage } from "./useDiffParser";

interface DiffHeaderProps {
	file: DiffFile;
	isViewed: boolean;
	isCollapsed: boolean;
	onToggleViewed: () => void;
	onToggleCollapsed: () => void;
}

export function DiffHeader({
	file,
	isViewed,
	isCollapsed,
	onToggleViewed,
	onToggleCollapsed,
}: DiffHeaderProps) {
	const fileName = getFileDisplayName(file);
	const language = getFileLanguage(file);

	return (
		<div
			className={`flex items-center gap-3 px-3 py-2 bg-muted/50 border-b border-border cursor-pointer hover:bg-muted/70 transition-colors ${
				isViewed ? "opacity-60" : ""
			}`}
			onClick={onToggleCollapsed}
			onKeyDown={(e) => {
				if (e.key === "Enter" || e.key === " ") {
					e.preventDefault();
					onToggleCollapsed();
				}
			}}
			role="button"
			tabIndex={0}
		>
			<span className="text-muted-foreground">
				{isCollapsed ? (
					<ChevronRight className="h-4 w-4" />
				) : (
					<ChevronDown className="h-4 w-4" />
				)}
			</span>

			<span className="font-mono text-sm text-foreground truncate flex-1">
				{fileName}
			</span>

			<div className="flex items-center gap-3 shrink-0">
				{file.addedLines > 0 && (
					<span className="text-green-500 text-xs font-mono">
						+{file.addedLines}
					</span>
				)}
				{file.deletedLines > 0 && (
					<span className="text-red-500 text-xs font-mono">
						-{file.deletedLines}
					</span>
				)}
				<span className="text-muted-foreground text-xs">{language}</span>

				<label
					className="flex items-center gap-1.5 text-xs text-muted-foreground cursor-pointer"
					onClick={(e) => e.stopPropagation()}
					onKeyDown={(e) => e.stopPropagation()}
				>
					<input
						type="checkbox"
						checked={isViewed}
						onChange={onToggleViewed}
						className="h-3.5 w-3.5 rounded border-border bg-background"
					/>
					Viewed
				</label>
			</div>
		</div>
	);
}
