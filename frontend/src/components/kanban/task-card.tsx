"use client"

import { cn } from "~/lib/utils"
import { Bot, Clock, User } from "lucide-react"
import type { Task } from "~/lib/mock-data"
import { Badge } from "~/components/ui/badge"

interface TaskCardProps {
  task: Task
  onClick: () => void
}

export function TaskCard({ task, onClick }: TaskCardProps) {
  const priorityColors = {
    low: "bg-muted text-muted-foreground",
    medium: "bg-chart-3/20 text-chart-3",
    high: "bg-destructive/20 text-destructive",
  }

  const isAutomated = task.assignee === "ai"
  const isActive = ["PLANNING", "IN_PROGRESS", "AI_REVIEW"].includes(task.status)

  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full rounded-lg border border-border bg-card p-3 text-left transition-all",
        "hover:border-primary/50 hover:shadow-md hover:shadow-primary/5",
        isActive && "border-primary/30",
      )}
    >
      <div className="mb-2 flex items-start justify-between gap-2">
        <h4 className="text-sm font-medium leading-tight line-clamp-2">{task.title}</h4>
        {isAutomated && isActive && (
          <div className="shrink-0 rounded-full bg-primary/20 p-1">
            <Bot className="size-3 text-primary animate-pulse" />
          </div>
        )}
      </div>
      <p className="mb-3 text-xs text-muted-foreground line-clamp-2">{task.description}</p>
      <div className="flex items-center justify-between">
        <Badge variant="secondary" className={cn("text-xs", priorityColors[task.priority])}>
          {task.priority}
        </Badge>
        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
          {isAutomated ? <Bot className="size-3" /> : <User className="size-3" />}
          <Clock className="size-3" />
          <span>{new Date(task.updatedAt).toLocaleDateString()}</span>
        </div>
      </div>
    </button>
  )
}
