import { useState } from "react";
import type { DiffFile } from "diff2html/lib/types";
import { DiffHeader } from "./DiffHeader";
import { DiffContent } from "./DiffContent";
import { getFileDisplayName } from "./useDiffParser";

interface DiffFileSectionProps {
	file: DiffFile;
	isViewed: boolean;
	onToggleViewed: (filePath: string, viewed: boolean) => void;
	defaultCollapsed?: boolean;
}

export function DiffFileSection({
	file,
	isViewed,
	onToggleViewed,
	defaultCollapsed = false,
}: DiffFileSectionProps) {
	const [collapsed, setCollapsed] = useState(defaultCollapsed);
	const filePath = getFileDisplayName(file);

	return (
		<div className="border-b border-border last:border-b-0">
			<DiffHeader
				file={file}
				isViewed={isViewed}
				isCollapsed={collapsed}
				onToggleCollapsed={() => setCollapsed(!collapsed)}
				onToggleViewed={() => onToggleViewed(filePath, !isViewed)}
			/>
			{!collapsed && <DiffContent file={file} />}
		</div>
	);
}
