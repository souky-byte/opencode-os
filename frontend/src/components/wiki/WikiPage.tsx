import { useEffect, useMemo } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeRaw from "rehype-raw";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism";
import { useGetWikiPage } from "@/api/generated/wiki/wiki";
import { Loader } from "@/components/ui/loader";
import { useWikiStore } from "@/stores/useWikiStore";
import { MermaidDiagram } from "./MermaidDiagram";
import { cn } from "@/lib/utils";
import type { WikiPageResponse } from "@/api/generated/model/wikiPageResponse";

export function WikiPage() {
	const { currentPageSlug, setCurrentPageSlug, structure } = useWikiStore();

	useEffect(() => {
		if (!currentPageSlug && structure) {
			setCurrentPageSlug(structure.slug);
		}
	}, [currentPageSlug, structure, setCurrentPageSlug]);

	const { data, isLoading, error } = useGetWikiPage(currentPageSlug ?? "", {
		query: {
			enabled: !!currentPageSlug,
		},
	});

	if (!currentPageSlug) {
		return (
			<div className="flex h-full items-center justify-center p-6">
				<p className="text-muted-foreground">Select a page from the sidebar</p>
			</div>
		);
	}

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<Loader />
			</div>
		);
	}

	if (error || data?.status !== 200) {
		return (
			<div className="flex h-full items-center justify-center p-6">
				<div className="text-center">
					<h3 className="text-lg font-medium">Page Not Found</h3>
					<p className="mt-2 text-sm text-muted-foreground">
						The requested page could not be loaded.
					</p>
				</div>
			</div>
		);
	}

	const page = data.data;

	return (
		<div className="p-6">
			<article className="prose prose-invert prose-sm max-w-none">
				<PageHeader page={page} />
				<MarkdownContent content={page.content} hasDiagrams={page.has_diagrams} />
				<PageFooter page={page} />
			</article>
		</div>
	);
}

function PageHeader({ page }: { page: WikiPageResponse }) {
	const { setCurrentPageSlug, setViewMode } = useWikiStore();

	return (
		<header className="mb-6 not-prose">
			<div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
				<span className="px-2 py-0.5 rounded bg-accent capitalize">{page.page_type}</span>
				<ImportanceBadge importance={page.importance} />
				{page.updated_at && <span>Updated {new Date(page.updated_at).toLocaleDateString()}</span>}
			</div>
			<h1 className="text-2xl font-bold">{page.title}</h1>
			{page.file_paths && page.file_paths.length > 0 && (
				<div className="mt-2 flex flex-wrap gap-1">
					{page.file_paths.map((path) => (
						<span
							key={path}
							className="text-xs px-2 py-0.5 rounded bg-primary/10 text-primary font-mono"
						>
							{path}
						</span>
					))}
				</div>
			)}
			{page.related_pages && page.related_pages.length > 0 && (
				<div className="mt-3 flex items-center gap-2 flex-wrap">
					<span className="text-xs text-muted-foreground">Related:</span>
					{page.related_pages.map((slug) => (
						<button
							key={slug}
							type="button"
							onClick={() => {
								setCurrentPageSlug(slug);
								setViewMode("page");
							}}
							className="text-xs px-2 py-0.5 rounded bg-accent hover:bg-accent/80 text-foreground transition-colors"
						>
							{slug}
						</button>
					))}
				</div>
			)}
		</header>
	);
}

function PageFooter({ page }: { page: WikiPageResponse }) {
	if (!page.source_citations || page.source_citations.length === 0) {
		return null;
	}

	return (
		<footer className="mt-8 pt-6 border-t border-border not-prose">
			<h3 className="text-sm font-semibold text-muted-foreground mb-3">Source Files Referenced</h3>
			<div className="flex flex-wrap gap-2">
				{page.source_citations.map((citation, idx) => (
					<CitationBadge key={`${citation.file_path}-${idx}`} citation={citation} />
				))}
			</div>
		</footer>
	);
}

function CitationBadge({
	citation,
}: {
	citation: { file_path: string; start_line?: number | null; end_line?: number | null };
}) {
	const formatCitation = () => {
		if (citation.start_line && citation.end_line) {
			if (citation.start_line === citation.end_line) {
				return `${citation.file_path}:${citation.start_line}`;
			}
			return `${citation.file_path}:${citation.start_line}-${citation.end_line}`;
		}
		if (citation.start_line) {
			return `${citation.file_path}:${citation.start_line}`;
		}
		return citation.file_path;
	};

	return (
		<span className="text-xs px-2 py-1 rounded bg-accent/50 text-foreground/80 font-mono">
			{formatCitation()}
		</span>
	);
}

function ImportanceBadge({ importance }: { importance: string }) {
	const colors: Record<string, string> = {
		high: "bg-red-500/20 text-red-400 border-red-500/30",
		medium: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
		low: "bg-blue-500/20 text-blue-400 border-blue-500/30",
	};

	const color = colors[importance] || colors.medium;

	return (
		<span className={cn("px-2 py-0.5 rounded border text-xs capitalize", color)}>{importance}</span>
	);
}

function MarkdownContent({ content, hasDiagrams }: { content: string; hasDiagrams: boolean }) {
	const parts = useMemo(() => {
		if (!hasDiagrams) {
			return [{ type: "markdown" as const, content }];
		}

		const result: Array<{ type: "markdown" | "mermaid"; content: string }> = [];
		const mermaidRegex = /```mermaid\n([\s\S]*?)```/g;
		let lastIndex = 0;
		let match: RegExpExecArray | null;

		while ((match = mermaidRegex.exec(content)) !== null) {
			if (match.index > lastIndex) {
				result.push({
					type: "markdown",
					content: content.slice(lastIndex, match.index),
				});
			}
			result.push({
				type: "mermaid",
				content: match[1].trim(),
			});
			lastIndex = match.index + match[0].length;
		}

		if (lastIndex < content.length) {
			result.push({
				type: "markdown",
				content: content.slice(lastIndex),
			});
		}

		return result;
	}, [content, hasDiagrams]);

	return (
		<div className="space-y-4">
			{parts.map((part, index) => {
				if (part.type === "mermaid") {
					return <MermaidDiagram key={`mermaid-${index}`} chart={part.content} />;
				}
				return <MarkdownRenderer key={`md-${index}`} content={part.content} />;
			})}
		</div>
	);
}

function MarkdownRenderer({ content }: { content: string }) {
	return (
		<Markdown
			remarkPlugins={[remarkGfm]}
			rehypePlugins={[rehypeRaw]}
			components={{
				h1: ({ node, ...props }) => <h1 className="text-2xl font-bold mt-8 mb-4" {...props} />,
				h2: ({ node, ...props }) => <h2 className="text-xl font-semibold mt-8 mb-3" {...props} />,
				h3: ({ node, ...props }) => <h3 className="text-lg font-semibold mt-6 mb-2" {...props} />,
				h4: ({ node, ...props }) => <h4 className="text-base font-semibold mt-4 mb-2" {...props} />,
				p: ({ node, ...props }) => <p className="my-2 leading-relaxed" {...props} />,
				a: ({ node, ...props }) => (
					<a className="text-primary hover:underline font-medium" {...props} />
				),
				ul: ({ node, ...props }) => <ul className="list-disc ml-6 my-2 space-y-1" {...props} />,
				ol: ({ node, ...props }) => <ol className="list-decimal ml-6 my-2 space-y-1" {...props} />,
				li: ({ node, ...props }) => <li className="pl-1" {...props} />,
				blockquote: ({ node, ...props }) => (
					<blockquote
						className="border-l-4 border-primary/20 pl-4 italic my-4 text-muted-foreground"
						{...props}
					/>
				),
				details: ({ node, ...props }) => (
					<details
						className="my-4 rounded-lg border border-border bg-muted/30 overflow-hidden group"
						{...props}
					/>
				),
				summary: ({ node, ...props }) => (
					<summary
						className="px-4 py-2 cursor-pointer font-medium bg-muted/50 hover:bg-muted/70 transition-colors select-none"
						{...props}
					/>
				),
				table: ({ node, ...props }) => (
					<div className="my-6 w-full overflow-y-auto rounded-lg border border-border">
						<table className="w-full text-sm" {...props} />
					</div>
				),
				thead: ({ node, ...props }) => <thead className="bg-muted/50" {...props} />,
				tbody: ({ node, ...props }) => (
					<tbody className="divide-y divide-border [&>tr:nth-child(even)]:bg-muted/30" {...props} />
				),
				tr: ({ node, ...props }) => (
					<tr className="transition-colors hover:bg-muted/50" {...props} />
				),
				th: ({ node, ...props }) => (
					<th
						className="h-10 px-4 text-left align-middle font-medium text-muted-foreground [&:has([role=checkbox])]:pr-0"
						{...props}
					/>
				),
				td: ({ node, ...props }) => (
					<td className="p-4 align-middle [&:has([role=checkbox])]:pr-0" {...props} />
				),
				code: ({ node, inline, className, children, ...props }: any) => {
					const match = /language-(\w+)/.exec(className || "");
					return !inline && match ? (
						<div className="my-4 rounded-md overflow-hidden">
							<SyntaxHighlighter
								style={vscDarkPlus}
								language={match[1]}
								PreTag="div"
								customStyle={{ margin: 0, borderRadius: 0 }}
								{...props}
							>
								{String(children).replace(/\n$/, "")}
							</SyntaxHighlighter>
						</div>
					) : (
						<code
							className="bg-muted px-1.5 py-0.5 rounded text-sm font-mono text-foreground"
							{...props}
						>
							{children}
						</code>
					);
				},
			}}
		>
			{content}
		</Markdown>
	);
}
