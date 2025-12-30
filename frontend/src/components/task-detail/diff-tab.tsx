"use client"

import { cn } from "~/lib/utils"

interface DiffTabProps {
  diff: string
}

export function DiffTab({ diff }: DiffTabProps) {
  const lines = diff.split("\n")

  return (
    <div className="p-4">
      <div className="rounded-lg border border-border overflow-hidden">
        <div className="bg-secondary px-4 py-2 border-b border-border">
          <span className="text-xs font-medium text-muted-foreground">2 files changed, +50 -3</span>
        </div>
        <pre className="text-xs font-mono overflow-x-auto">
          {lines.map((line, index) => (
            <div
              key={index}
              className={cn(
                "px-4 py-0.5",
                line.startsWith("+") && !line.startsWith("+++") && "bg-primary/10 text-primary",
                line.startsWith("-") && !line.startsWith("---") && "bg-destructive/10 text-destructive",
                line.startsWith("@@") && "bg-chart-2/10 text-chart-2",
                line.startsWith("diff") && "bg-secondary text-muted-foreground font-semibold pt-2",
              )}
            >
              {line}
            </div>
          ))}
        </pre>
      </div>
    </div>
  )
}
