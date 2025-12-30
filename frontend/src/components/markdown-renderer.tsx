"use client";

import type React from "react";

import { cn } from "~/lib/utils";

interface MarkdownRendererProps {
	content: string;
	className?: string;
}

export function MarkdownRenderer({ content, className }: MarkdownRendererProps) {
	const renderMarkdown = (text: string) => {
		const lines = text.split("\n");
		const elements: React.ReactNode[] = [];
		let inCodeBlock = false;
		let codeBlockContent: string[] = [];
		let listItems: string[] = [];
		let listType: "ul" | "ol" | null = null;

		const flushList = () => {
			if (listItems.length > 0 && listType) {
				const ListTag = listType;
				elements.push(
					<ListTag
						key={elements.length}
						className={cn("my-3 ml-6 space-y-1", listType === "ul" ? "list-disc" : "list-decimal")}
					>
						{listItems.map((item, i) => (
							<li key={i} className="text-muted-foreground text-sm">
								{renderInline(item)}
							</li>
						))}
					</ListTag>,
				);
				listItems = [];
				listType = null;
			}
		};

		const renderInline = (line: string): React.ReactNode => {
			// Process inline elements
			const parts: React.ReactNode[] = [];
			let remaining = line;
			let key = 0;

			while (remaining.length > 0) {
				// Checkbox - [x] or [ ]
				const checkboxMatch = remaining.match(/^\[(x| )\]\s*/);
				if (checkboxMatch) {
					const checked = checkboxMatch[1] === "x";
					parts.push(
						<span key={key++} className="inline-flex items-center gap-1.5 mr-1">
							<span
								className={cn(
									"size-4 rounded border flex items-center justify-center text-xs",
									checked
										? "bg-primary border-primary text-primary-foreground"
										: "border-muted-foreground",
								)}
							>
								{checked && "✓"}
							</span>
						</span>,
					);
					remaining = remaining.slice(checkboxMatch[0].length);
					continue;
				}

				// Inline code
				const codeMatch = remaining.match(/^`([^`]+)`/);
				if (codeMatch) {
					parts.push(
						<code
							key={key++}
							className="px-1.5 py-0.5 rounded bg-secondary text-primary font-mono text-xs"
						>
							{codeMatch[1]}
						</code>,
					);
					remaining = remaining.slice(codeMatch[0].length);
					continue;
				}

				// Bold
				const boldMatch = remaining.match(/^\*\*([^*]+)\*\*/);
				if (boldMatch) {
					parts.push(
						<strong key={key++} className="font-semibold text-foreground">
							{boldMatch[1]}
						</strong>,
					);
					remaining = remaining.slice(boldMatch[0].length);
					continue;
				}

				// Italic
				const italicMatch = remaining.match(/^\*([^*]+)\*/);
				if (italicMatch) {
					parts.push(
						<em key={key++} className="italic">
							{italicMatch[1]}
						</em>,
					);
					remaining = remaining.slice(italicMatch[0].length);
					continue;
				}

				// Link
				const linkMatch = remaining.match(/^\[([^\]]+)\]$$([^)]+)$$/);
				if (linkMatch) {
					parts.push(
						<a
							key={key++}
							href={linkMatch[2]}
							className="text-primary hover:underline"
							target="_blank"
							rel="noopener noreferrer"
						>
							{linkMatch[1]}
						</a>,
					);
					remaining = remaining.slice(linkMatch[0].length);
					continue;
				}

				// Regular text - find next special character
				const nextSpecial = remaining.search(/[`*[]/);
				if (nextSpecial === -1) {
					parts.push(remaining);
					break;
				} else if (nextSpecial === 0) {
					parts.push(remaining[0]);
					remaining = remaining.slice(1);
				} else {
					parts.push(remaining.slice(0, nextSpecial));
					remaining = remaining.slice(nextSpecial);
				}
			}

			return parts;
		};

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			if (line === undefined) continue;

			// Code block handling
			if (line.startsWith("```")) {
				if (inCodeBlock) {
					elements.push(
						<pre
							key={elements.length}
							className="my-3 p-4 rounded-lg bg-secondary/50 border border-border overflow-x-auto"
						>
							<code className="text-xs font-mono text-foreground">
								{codeBlockContent.join("\n")}
							</code>
						</pre>,
					);
					codeBlockContent = [];
					inCodeBlock = false;
				} else {
					flushList();
					inCodeBlock = true;
				}
				continue;
			}

			if (inCodeBlock) {
				codeBlockContent.push(line);
				continue;
			}

			// Headers
			const h1Match = line.match(/^# (.+)/);
			if (h1Match) {
				flushList();
				elements.push(
					<h1
						key={elements.length}
						className="text-xl font-bold text-foreground mt-6 mb-3 first:mt-0"
					>
						{renderInline(h1Match[1]!)}
					</h1>,
				);
				continue;
			}

			const h2Match = line.match(/^## (.+)/);
			if (h2Match) {
				flushList();
				elements.push(
					<h2 key={elements.length} className="text-lg font-semibold text-foreground mt-5 mb-2">
						{renderInline(h2Match[1]!)}
					</h2>,
				);
				continue;
			}

			const h3Match = line.match(/^### (.+)/);
			if (h3Match) {
				flushList();
				elements.push(
					<h3 key={elements.length} className="text-base font-semibold text-foreground mt-4 mb-2">
						{renderInline(h3Match[1]!)}
					</h3>,
				);
				continue;
			}

			// Horizontal rule
			if (line.match(/^(-{3,}|_{3,}|\*{3,})$/)) {
				flushList();
				elements.push(<hr key={elements.length} className="my-4 border-border" />);
				continue;
			}

			// Numbered list
			const numberedMatch = line.match(/^(\d+)\.\s+(.+)/);
			if (numberedMatch) {
				if (listType !== "ol") {
					flushList();
					listType = "ol";
				}
				listItems.push(numberedMatch[2]!);
				continue;
			}

			// Unordered list
			const bulletMatch = line.match(/^[-*]\s+(.+)/);
			if (bulletMatch) {
				if (listType !== "ul") {
					flushList();
					listType = "ul";
				}
				listItems.push(bulletMatch[1]!);
				continue;
			}

			// Empty line
			if (line.trim() === "") {
				flushList();
				continue;
			}

			// Regular paragraph
			flushList();
			elements.push(
				<p key={elements.length} className="text-sm text-muted-foreground my-2 leading-relaxed">
					{renderInline(line)}
				</p>,
			);
		}

		flushList();
		return elements;
	};

	return (
		<div className={cn("prose prose-sm max-w-none", className)}>{renderMarkdown(content)}</div>
	);
}
