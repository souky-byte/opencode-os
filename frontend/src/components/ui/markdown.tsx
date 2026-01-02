import { useMemo } from "react";
import { cn } from "@/lib/utils";

interface MarkdownProps {
	text: string;
	className?: string;
}

// Simple markdown parser for basic formatting
// For production, consider using react-markdown or marked
function parseMarkdown(text: string): string {
	if (!text) return "";

	let html = text
		// Escape HTML
		.replace(/&/g, "&amp;")
		.replace(/</g, "&lt;")
		.replace(/>/g, "&gt;")
		// Code blocks
		.replace(
			/```(\w+)?\n([\s\S]*?)```/g,
			'<pre class="bg-muted/50 rounded-md p-3 overflow-x-auto my-2"><code class="text-sm font-mono">$2</code></pre>',
		)
		// Inline code
		.replace(
			/`([^`]+)`/g,
			'<code class="bg-muted px-1.5 py-0.5 rounded text-sm font-mono text-primary">$1</code>',
		)
		// Bold
		.replace(
			/\*\*([^*]+)\*\*/g,
			'<strong class="font-semibold text-foreground">$1</strong>',
		)
		// Italic
		.replace(/\*([^*]+)\*/g, '<em class="italic">$1</em>')
		// Headers
		.replace(
			/^### (.+)$/gm,
			'<h3 class="text-base font-semibold text-foreground mt-4 mb-2">$1</h3>',
		)
		.replace(
			/^## (.+)$/gm,
			'<h2 class="text-lg font-semibold text-foreground mt-4 mb-2">$1</h2>',
		)
		.replace(
			/^# (.+)$/gm,
			'<h1 class="text-xl font-bold text-foreground mt-4 mb-2">$1</h1>',
		)
		// Lists
		.replace(
			/^- (.+)$/gm,
			'<li class="ml-4 list-disc text-muted-foreground">$1</li>',
		)
		.replace(
			/^\d+\. (.+)$/gm,
			'<li class="ml-4 list-decimal text-muted-foreground">$1</li>',
		)
		// Links
		.replace(
			/\[([^\]]+)\]\(([^)]+)\)/g,
			'<a href="$2" class="text-primary hover:underline" target="_blank" rel="noopener noreferrer">$1</a>',
		)
		// Horizontal rule
		.replace(/^---$/gm, '<hr class="border-border my-4" />')
		// Paragraphs (double newlines)
		.replace(/\n\n/g, '</p><p class="my-2">')
		// Single newlines within paragraphs
		.replace(/\n/g, "<br />");

	// Wrap in paragraph
	html = `<p class="my-2">${html}</p>`;

	// Clean up empty paragraphs
	html = html.replace(/<p class="my-2"><\/p>/g, "");

	return html;
}

export function Markdown({ text, className }: MarkdownProps) {
	const html = useMemo(() => parseMarkdown(text), [text]);

	return (
		<div
			className={cn(
				"prose prose-sm prose-invert max-w-none",
				"text-sm text-muted-foreground leading-relaxed",
				"[&_strong]:text-foreground [&_em]:text-foreground/90",
				"[&_h1]:text-foreground [&_h2]:text-foreground [&_h3]:text-foreground",
				"[&_a]:text-primary [&_a:hover]:underline",
				"[&_code]:text-primary [&_pre]:bg-muted/50",
				"[&_li]:text-muted-foreground [&_li]:marker:text-muted-foreground/50",
				className,
			)}
			// biome-ignore lint: using dangerouslySetInnerHTML for markdown
			dangerouslySetInnerHTML={{ __html: html }}
		/>
	);
}

// Simple text truncator with expand
export function TruncatedText({
	text,
	maxLength = 200,
	className,
}: { text: string; maxLength?: number; className?: string }) {
	const shouldTruncate = text.length > maxLength;
	const truncated = shouldTruncate ? `${text.slice(0, maxLength)}...` : text;

	return (
		<span className={cn("text-sm text-muted-foreground", className)}>
			{truncated}
		</span>
	);
}
