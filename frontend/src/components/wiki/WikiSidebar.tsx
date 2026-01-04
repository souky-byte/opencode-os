import { useState } from "react";
import { cn } from "@/lib/utils";
import { useWikiStore, type WikiTreeNode } from "@/stores/useWikiStore";

export function WikiSidebar() {
	const { structure, currentPageSlug, setCurrentPageSlug, setViewMode } = useWikiStore();

	const handlePageSelect = (slug: string) => {
		setCurrentPageSlug(slug);
		setViewMode("page");
	};

	if (!structure) {
		return (
			<div className="w-64 border-r border-border bg-card/50 p-4">
				<div className="text-sm text-muted-foreground">No structure available</div>
			</div>
		);
	}

	return (
		<aside className="w-64 flex-shrink-0 border-r border-border bg-card/50 overflow-y-auto">
			<div className="p-3">
				<h2 className="px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
					Documentation
				</h2>
				<nav className="mt-2 space-y-0.5">
					<TreeNode
						node={structure}
						currentSlug={currentPageSlug}
						onSelect={handlePageSelect}
						level={0}
					/>
				</nav>
			</div>
		</aside>
	);
}

function TreeNode({
	node,
	currentSlug,
	onSelect,
	level,
}: {
	node: WikiTreeNode;
	currentSlug: string | null;
	onSelect: (slug: string) => void;
	level: number;
}) {
	const [isExpanded, setIsExpanded] = useState(level < 2);
	const hasChildren = node.children && node.children.length > 0;
	const isSelected = currentSlug === node.slug;

	const handleClick = () => {
		onSelect(node.slug);
		if (hasChildren) {
			setIsExpanded(!isExpanded);
		}
	};

	return (
		<div>
			<button
				type="button"
				onClick={handleClick}
				className={cn(
					"flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-sm transition-colors",
					isSelected ? "bg-primary/10 text-primary font-medium" : "text-foreground hover:bg-accent",
				)}
				style={{ paddingLeft: `${level * 12 + 8}px` }}
			>
				{hasChildren && (
					<span className="flex h-4 w-4 items-center justify-center text-muted-foreground">
						<svg
							className={cn("h-3 w-3 transition-transform", isExpanded && "rotate-90")}
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<path d="M9 18l6-6-6-6" />
						</svg>
					</span>
				)}
				{!hasChildren && <span className="w-4" />}
				<PageTypeIcon type={node.page_type} />
				<span className="truncate">{node.title}</span>
			</button>
			{hasChildren && isExpanded && (
				<div>
					{node.children.map((child) => (
						<TreeNode
							key={child.slug}
							node={child}
							currentSlug={currentSlug}
							onSelect={onSelect}
							level={level + 1}
						/>
					))}
				</div>
			)}
		</div>
	);
}

function PageTypeIcon({ type }: { type: string }) {
	const iconClass = "h-4 w-4 text-muted-foreground";

	switch (type) {
		case "overview":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<path d="M3 9l9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
					<polyline points="9 22 9 12 15 12 15 22" />
				</svg>
			);
		case "module":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
				</svg>
			);
		case "file":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
					<polyline points="14 2 14 8 20 8" />
					<line x1="16" y1="13" x2="8" y2="13" />
					<line x1="16" y1="17" x2="8" y2="17" />
					<polyline points="10 9 9 9 8 9" />
				</svg>
			);
		default:
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
					<polyline points="14 2 14 8 20 8" />
				</svg>
			);
	}
}
