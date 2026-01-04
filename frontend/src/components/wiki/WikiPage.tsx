import { useEffect, useMemo } from "react";
import { useGetWikiPage } from "@/api/generated/wiki/wiki";
import { Loader } from "@/components/ui/loader";
import { useWikiStore } from "@/stores/useWikiStore";
import { MermaidDiagram } from "./MermaidDiagram";

export function WikiPage() {
	const { currentPageSlug, setCurrentPageSlug, structure } = useWikiStore();

	// Auto-select the first page (overview) if none selected
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
				<header className="mb-6 not-prose">
					<div className="flex items-center gap-2 text-xs text-muted-foreground mb-2">
						<span className="px-2 py-0.5 rounded bg-accent capitalize">{page.page_type}</span>
						{page.updated_at && (
							<span>Updated {new Date(page.updated_at).toLocaleDateString()}</span>
						)}
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
				</header>

				<MarkdownContent content={page.content} hasDiagrams={page.has_diagrams} />
			</article>
		</div>
	);
}

function MarkdownContent({ content, hasDiagrams }: { content: string; hasDiagrams: boolean }) {
	// Parse content and extract mermaid diagrams
	const parts = useMemo(() => {
		if (!hasDiagrams) {
			return [{ type: "markdown" as const, content }];
		}

		const result: Array<{ type: "markdown" | "mermaid"; content: string }> = [];
		const mermaidRegex = /```mermaid\n([\s\S]*?)```/g;
		let lastIndex = 0;
		let match: RegExpExecArray | null;

		while ((match = mermaidRegex.exec(content)) !== null) {
			// Add markdown before this diagram
			if (match.index > lastIndex) {
				result.push({
					type: "markdown",
					content: content.slice(lastIndex, match.index),
				});
			}
			// Add the diagram
			result.push({
				type: "mermaid",
				content: match[1].trim(),
			});
			lastIndex = match.index + match[0].length;
		}

		// Add remaining markdown
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
	// Simple markdown rendering - in production, use a proper markdown parser
	const html = useMemo(() => {
		let parsed = content;

		// Headers
		parsed = parsed.replace(/^### (.*$)/gm, '<h3 class="text-lg font-semibold mt-6 mb-2">$1</h3>');
		parsed = parsed.replace(/^## (.*$)/gm, '<h2 class="text-xl font-semibold mt-8 mb-3">$1</h2>');
		parsed = parsed.replace(/^# (.*$)/gm, '<h1 class="text-2xl font-bold mt-8 mb-4">$1</h1>');

		// Code blocks
		parsed = parsed.replace(
			/```(\w+)?\n([\s\S]*?)```/g,
			'<pre class="bg-accent rounded-md p-4 overflow-x-auto my-4"><code class="text-sm font-mono">$2</code></pre>',
		);

		// Inline code
		parsed = parsed.replace(
			/`([^`]+)`/g,
			'<code class="bg-accent px-1.5 py-0.5 rounded text-sm font-mono">$1</code>',
		);

		// Bold
		parsed = parsed.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");

		// Italic
		parsed = parsed.replace(/\*([^*]+)\*/g, "<em>$1</em>");

		// Links
		parsed = parsed.replace(
			/\[([^\]]+)\]\(([^)]+)\)/g,
			'<a href="$2" class="text-primary hover:underline">$1</a>',
		);

		// Lists
		parsed = parsed.replace(/^- (.*$)/gm, '<li class="ml-4">$1</li>');
		parsed = parsed.replace(/(<li.*<\/li>\n?)+/g, '<ul class="list-disc my-2">$&</ul>');

		// Paragraphs
		parsed = parsed.replace(/^(?!<[h|p|u|l|o|d|pre])(.*[^\n])$/gm, '<p class="my-2">$1</p>');

		return parsed;
	}, [content]);

	return <div dangerouslySetInnerHTML={{ __html: html }} />;
}
