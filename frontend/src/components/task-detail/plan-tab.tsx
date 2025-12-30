"use client"

import { MarkdownRenderer } from "~/components/markdown-renderer"

interface PlanTabProps {
  content: string
}

export function PlanTab({ content }: PlanTabProps) {
  return (
    <div className="p-4">
      <div className="rounded-lg border border-border bg-secondary/30 p-4">
        <MarkdownRenderer content={content} />
      </div>
    </div>
  )
}
