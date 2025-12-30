"use client"

import { TaskCard } from "./task-card"
import { KanbanColumn } from "./kanban-column"
import type { Task, TaskStatus } from "~/lib/mock-data"

interface KanbanViewProps {
  tasks: Task[]
  onTaskSelect: (task: Task) => void
  onTaskStatusChange: (taskId: string, newStatus: TaskStatus) => void
}

const columns: { status: TaskStatus; label: string; color: string }[] = [
  { status: "TODO", label: "Todo", color: "bg-muted-foreground" },
  { status: "PLANNING", label: "Planning", color: "bg-chart-2" },
  { status: "IN_PROGRESS", label: "In Progress", color: "bg-chart-1" },
  { status: "AI_REVIEW", label: "AI Review", color: "bg-chart-3" },
  { status: "REVIEW", label: "Review", color: "bg-chart-5" },
  { status: "DONE", label: "Done", color: "bg-primary" },
]

export function KanbanView({ tasks, onTaskSelect, onTaskStatusChange }: KanbanViewProps) {
  const getTasksByStatus = (status: TaskStatus) => tasks.filter((task) => task.status === status)

  return (
    <div className="h-full p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Kanban Board</h1>
          <p className="text-sm text-muted-foreground">
            {tasks.length} tasks • {tasks.filter((t) => t.assignee === "ai").length} automated
          </p>
        </div>
      </div>
      <div className="flex gap-4 overflow-x-auto pb-4">
        {columns.map((column) => (
          <KanbanColumn
            key={column.status}
            status={column.status}
            label={column.label}
            color={column.color}
            count={getTasksByStatus(column.status).length}
          >
            {getTasksByStatus(column.status).map((task) => (
              <TaskCard key={task.id} task={task} onClick={() => onTaskSelect(task)} />
            ))}
          </KanbanColumn>
        ))}
      </div>
    </div>
  )
}
