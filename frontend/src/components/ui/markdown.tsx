import { useMemo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";
import { cn } from "@/lib/utils";

interface MarkdownProps {
  text?: string;
  children?: string;
  className?: string;
}

// Remove HTML comments (used by GitHub bots like CodeRabbit)
function removeHtmlComments(content: string): string {
  // Remove HTML comments including multiline ones
  return content.replace(/<!--[\s\S]*?-->/g, "");
}

export function Markdown({ text, children, className }: MarkdownProps) {
  const rawContent = text ?? children ?? "";

  if (!rawContent) {
    return null;
  }

  // Clean up HTML comments before rendering
  const content = useMemo(() => removeHtmlComments(rawContent), [rawContent]);

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
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          code({ className: codeClassName, children: codeChildren, ...props }) {
            const match = /language-(\w+)/.exec(codeClassName || "");
            const isInline = !match && !codeClassName;

            if (isInline) {
              return (
                <code
                  className="bg-muted px-1.5 py-0.5 rounded text-sm font-mono text-primary"
                  {...props}
                >
                  {codeChildren}
                </code>
              );
            }

            return (
              <SyntaxHighlighter
                style={oneDark}
                language={match?.[1] || "text"}
                PreTag="div"
                customStyle={{
                  margin: 0,
                  borderRadius: "0.375rem",
                  fontSize: "0.75rem",
                }}
              >
                {String(codeChildren).replace(/\n$/, "")}
              </SyntaxHighlighter>
            );
          },
          pre({ children: preChildren }) {
            return <>{preChildren}</>;
          },
          a({ href, children: linkChildren }) {
            return (
              <a
                href={href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-primary hover:underline"
              >
                {linkChildren}
              </a>
            );
          },
          p({ children: pChildren }) {
            return <p className="my-2">{pChildren}</p>;
          },
          ul({ children: ulChildren }) {
            return <ul className="my-2 ml-4 list-disc">{ulChildren}</ul>;
          },
          ol({ children: olChildren }) {
            return <ol className="my-2 ml-4 list-decimal">{olChildren}</ol>;
          },
          li({ children: liChildren }) {
            return <li className="text-muted-foreground">{liChildren}</li>;
          },
          blockquote({ children: bqChildren }) {
            return (
              <blockquote className="border-l-2 border-muted-foreground/30 pl-3 italic text-muted-foreground my-2">
                {bqChildren}
              </blockquote>
            );
          },
          h1({ children: h1Children }) {
            return (
              <h1 className="text-xl font-bold text-foreground mt-4 mb-2">
                {h1Children}
              </h1>
            );
          },
          h2({ children: h2Children }) {
            return (
              <h2 className="text-lg font-semibold text-foreground mt-4 mb-2">
                {h2Children}
              </h2>
            );
          },
          h3({ children: h3Children }) {
            return (
              <h3 className="text-base font-semibold text-foreground mt-4 mb-2">
                {h3Children}
              </h3>
            );
          },
          hr() {
            return <hr className="border-border my-4" />;
          },
          // GFM: Tables
          table({ children: tableChildren }) {
            return (
              <div className="my-4 overflow-x-auto">
                <table className="min-w-full border-collapse border border-border text-sm">
                  {tableChildren}
                </table>
              </div>
            );
          },
          thead({ children: theadChildren }) {
            return <thead className="bg-muted/50">{theadChildren}</thead>;
          },
          tbody({ children: tbodyChildren }) {
            return <tbody>{tbodyChildren}</tbody>;
          },
          tr({ children: trChildren }) {
            return <tr className="border-b border-border">{trChildren}</tr>;
          },
          th({ children: thChildren }) {
            return (
              <th className="px-3 py-2 text-left font-semibold text-foreground border border-border">
                {thChildren}
              </th>
            );
          },
          td({ children: tdChildren }) {
            return (
              <td className="px-3 py-2 text-muted-foreground border border-border">
                {tdChildren}
              </td>
            );
          },
          // GFM: Strikethrough
          del({ children: delChildren }) {
            return (
              <del className="line-through text-muted-foreground/60">
                {delChildren}
              </del>
            );
          },
          // GFM: Task list items
          input({ checked, ...props }) {
            return (
              <input
                type="checkbox"
                checked={checked}
                disabled
                className="mr-2 accent-primary"
                {...props}
              />
            );
          },
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}

// Simple text truncator with expand
export function TruncatedText({
  text,
  maxLength = 200,
  className,
}: {
  text: string;
  maxLength?: number;
  className?: string;
}) {
  const shouldTruncate = text.length > maxLength;
  const truncated = useMemo(
    () => (shouldTruncate ? `${text.slice(0, maxLength)}...` : text),
    [text, maxLength, shouldTruncate],
  );

  return (
    <span className={cn("text-sm text-muted-foreground", className)}>
      {truncated}
    </span>
  );
}
