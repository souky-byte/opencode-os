import { useState } from "react";
import { cn } from "@/lib/utils";
import { useWikiStore, type WikiTreeNode, type WikiSection } from "@/stores/useWikiStore";

export function WikiSidebar() {
	const { structure, sections, currentPageSlug, setCurrentPageSlug, setViewMode } = useWikiStore();

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

	const hasSections = sections && sections.length > 0;

	return (
		<aside className="w-64 flex-shrink-0 border-r border-border bg-card/50 overflow-y-auto">
			<div className="p-3">
				<h2 className="px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
					Documentation
				</h2>
				<nav className="mt-2 space-y-1">
					{hasSections ? (
						<SectionNavigation
							sections={sections}
							structure={structure}
							currentSlug={currentPageSlug}
							onSelect={handlePageSelect}
						/>
					) : (
						<TreeNode
							node={structure}
							currentSlug={currentPageSlug}
							onSelect={handlePageSelect}
							level={0}
						/>
					)}
				</nav>
			</div>
		</aside>
	);
}

function SectionNavigation({
	sections,
	structure,
	currentSlug,
	onSelect,
}: {
	sections: WikiSection[];
	structure: WikiTreeNode;
	currentSlug: string | null;
	onSelect: (slug: string) => void;
}) {
	const sortedSections = [...sections].sort((a, b) => a.order - b.order);

	const findPageInTree = (slug: string, node: WikiTreeNode): WikiTreeNode | null => {
		if (node.slug === slug) return node;
		for (const child of node.children || []) {
			const found = findPageInTree(slug, child);
			if (found) return found;
		}
		return null;
	};

	const pagesInSections = new Set(sections.flatMap((s) => s.page_slugs));
	const orphanPages = collectOrphanPages(structure, pagesInSections);

	return (
		<div className="space-y-2">
			{sortedSections.map((section) => (
				<SectionGroup
					key={section.id}
					section={section}
					structure={structure}
					currentSlug={currentSlug}
					onSelect={onSelect}
					findPage={findPageInTree}
				/>
			))}

			{orphanPages.length > 0 && (
				<div className="pt-2 border-t border-border/50">
					<div className="px-2 py-1 text-xs font-medium text-muted-foreground">Other Pages</div>
					{orphanPages.map((page) => (
						<PageButton
							key={page.slug}
							page={page}
							isSelected={currentSlug === page.slug}
							onSelect={onSelect}
							level={0}
						/>
					))}
				</div>
			)}
		</div>
	);
}

function collectOrphanPages(node: WikiTreeNode, assignedSlugs: Set<string>): WikiTreeNode[] {
	const orphans: WikiTreeNode[] = [];

	const traverse = (n: WikiTreeNode) => {
		if (!assignedSlugs.has(n.slug)) {
			orphans.push(n);
		}
		for (const child of n.children || []) {
			traverse(child);
		}
	};

	traverse(node);
	return orphans;
}

function SectionGroup({
	section,
	structure,
	currentSlug,
	onSelect,
	findPage,
}: {
	section: WikiSection;
	structure: WikiTreeNode;
	currentSlug: string | null;
	onSelect: (slug: string) => void;
	findPage: (slug: string, node: WikiTreeNode) => WikiTreeNode | null;
}) {
	const hasSelectedPage = section.page_slugs.includes(currentSlug ?? "");
	const [isExpanded, setIsExpanded] = useState(hasSelectedPage || section.order === 0);

	const pages = section.page_slugs
		.map((slug) => findPage(slug, structure))
		.filter((p): p is WikiTreeNode => p !== null);

	if (pages.length === 0) return null;

	return (
		<div>
			<button
				type="button"
				onClick={() => setIsExpanded(!isExpanded)}
				className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm font-medium text-foreground hover:bg-accent transition-colors"
			>
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
				<SectionIcon sectionId={section.id} />
				<span className="truncate">{section.title}</span>
				<span className="ml-auto text-xs text-muted-foreground">{pages.length}</span>
			</button>

			{isExpanded && (
				<div className="ml-2 border-l border-border/50 pl-2 mt-0.5">
					{pages.map((page) => (
						<PageButton
							key={page.slug}
							page={page}
							isSelected={currentSlug === page.slug}
							onSelect={onSelect}
							level={1}
						/>
					))}
				</div>
			)}
		</div>
	);
}

function PageButton({
	page,
	isSelected,
	onSelect,
	level,
}: {
	page: WikiTreeNode;
	isSelected: boolean;
	onSelect: (slug: string) => void;
	level: number;
}) {
	return (
		<button
			type="button"
			onClick={() => onSelect(page.slug)}
			className={cn(
				"flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-left text-sm transition-colors",
				isSelected
					? "bg-primary/10 text-primary font-medium"
					: "text-foreground/80 hover:bg-accent",
			)}
			style={{ paddingLeft: `${level * 8 + 8}px` }}
		>
			<PageTypeIcon type={page.page_type} />
			<span className="truncate">{page.title}</span>
		</button>
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

function SectionIcon({ sectionId }: { sectionId: string }) {
	const iconClass = "h-4 w-4 text-muted-foreground";

	switch (sectionId) {
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
		case "architecture":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<rect x="3" y="3" width="7" height="7" />
					<rect x="14" y="3" width="7" height="7" />
					<rect x="14" y="14" width="7" height="7" />
					<rect x="3" y="14" width="7" height="7" />
				</svg>
			);
		case "core-features":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
				</svg>
			);
		case "backend":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
					<line x1="8" y1="21" x2="16" y2="21" />
					<line x1="12" y1="17" x2="12" y2="21" />
				</svg>
			);
		case "frontend":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
					<line x1="12" y1="18" x2="12" y2="18" />
				</svg>
			);
		case "data-flow":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<ellipse cx="12" cy="5" rx="9" ry="3" />
					<path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3" />
					<path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
				</svg>
			);
		case "deployment":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<path d="M22.61 16.95A5 5 0 0018 10h-1.26A8 8 0 103 16.29" />
					<polyline points="16 16 12 12 8 16" />
					<line x1="12" y1="12" x2="12" y2="21" />
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
					<path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
				</svg>
			);
	}
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
		case "architecture":
			return (
				<svg
					className={iconClass}
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<rect x="3" y="3" width="7" height="7" />
					<rect x="14" y="3" width="7" height="7" />
					<rect x="14" y="14" width="7" height="7" />
					<rect x="3" y="14" width="7" height="7" />
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
