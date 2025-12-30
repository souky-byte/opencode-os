"use client"

import type React from "react"

import { cn } from "~/lib/utils"
import type { TaskStatus } from "~/lib/mock-data"

interface KanbanColumnProps {
  status: TaskStatus
  label: string
  color: string
  count: number
  children: React.ReactNode
}

export function KanbanColumn({ label, color, count, children }: KanbanColumnProps) {
  return (
    <div className="flex w-72 shrink-0 flex-col">
      <div className="mb-3 flex items-center gap-2">
        <div className={cn("size-2.5 rounded-full", color)} />
        <h3 className="text-sm font-medium">{label}</h3>
        <span className="ml-auto rounded-full bg-secondary px-2 py-0.5 text-xs font-medium text-muted-foreground">
          {count}
        </span>
      </div>
      <div className="flex flex-col gap-2 rounded-lg bg-secondary/30 p-2 min-h-[200px]">{children}</div>
    </div>
  )
}
