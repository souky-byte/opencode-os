"use client"

import { MarkdownRenderer } from "~/components/markdown-renderer"

interface ReviewTabProps {
  content: string
}

export function ReviewTab({ content }: ReviewTabProps) {
  return (
    <div className="p-4">
      <div className="rounded-lg border border-border bg-secondary/30 p-4">
        <MarkdownRenderer content={content} />
      </div>
    </div>
  )
}
