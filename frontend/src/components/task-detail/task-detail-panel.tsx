"use client"

import { useState } from "react"
import { Button } from "~/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs"
import { Badge } from "~/components/ui/badge"
import { Check, ChevronRight, FileText, GitCompare, MessageSquare, Play, RotateCcw, Server, X } from "lucide-react"
import type { Task, TaskStatus } from "~/lib/mock-data"
import { mockDiff, mockPlan, mockReview, mockSessions } from "~/lib/mock-data"
import { cn } from "~/lib/utils"
import { PlanTab } from "./plan-tab"
import { DiffTab } from "./diff-tab"
import { ReviewTab } from "./review-tab"
import { DevServerTab } from "./dev-server-tab"
import { SessionTab } from "./session-tab"

interface TaskDetailPanelProps {
  task: Task
  onClose: () => void
  onStatusChange: (taskId: string, newStatus: TaskStatus) => void
}

const statusFlow: TaskStatus[] = ["TODO", "PLANNING", "IN_PROGRESS", "AI_REVIEW", "REVIEW", "DONE"]

export function TaskDetailPanel({ task, onClose, onStatusChange }: TaskDetailPanelProps) {
  const [activeTab, setActiveTab] = useState("plan")
  const currentSession = mockSessions.find((s) => s.taskId === task.id)

  const statusIndex = statusFlow.indexOf(task.status)
  const canApprove = task.status === "REVIEW"
  const canReject = task.status === "REVIEW"

  const handleApprove = () => {
    onStatusChange(task.id, "DONE")
  }

  const handleReject = () => {
    onStatusChange(task.id, "IN_PROGRESS")
  }

  return (
    <div className="fixed right-0 top-14 bottom-0 w-[480px] border-l border-border bg-background flex flex-col">
      <div className="flex items-center justify-between border-b border-border p-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Badge
              variant="outline"
              className={cn(
                "text-xs",
                task.status === "DONE" && "border-primary text-primary",
                task.status === "REVIEW" && "border-chart-5 text-chart-5",
                ["PLANNING", "IN_PROGRESS", "AI_REVIEW"].includes(task.status) && "border-chart-1 text-chart-1",
              )}
            >
              {task.status.replace("_", " ")}
            </Badge>
            <span className="text-xs text-muted-foreground font-mono">{task.id}</span>
          </div>
          <h2 className="text-lg font-semibold truncate">{task.title}</h2>
        </div>
        <Button variant="ghost" size="icon" onClick={onClose}>
          <X className="size-4" />
        </Button>
      </div>

      {/* Status Progress */}
      <div className="border-b border-border p-4">
        <div className="flex items-center justify-between text-xs mb-2">
          <span className="text-muted-foreground">Progress</span>
          <span className="font-medium">
            {statusIndex + 1} / {statusFlow.length}
          </span>
        </div>
        <div className="flex items-center gap-1">
          {statusFlow.map((status, index) => (
            <div key={status} className="flex items-center flex-1">
              <div className={cn("h-1.5 flex-1 rounded-full", index <= statusIndex ? "bg-primary" : "bg-secondary")} />
              {index < statusFlow.length - 1 && (
                <ChevronRight
                  className={cn(
                    "size-3 shrink-0 mx-0.5",
                    index < statusIndex ? "text-primary" : "text-muted-foreground",
                  )}
                />
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col overflow-hidden">
        <TabsList className="w-full justify-start rounded-none border-b border-border bg-transparent px-4 h-auto py-0">
          <TabsTrigger
            value="plan"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <FileText className="size-3.5" />
            Plan
          </TabsTrigger>
          <TabsTrigger
            value="diff"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <GitCompare className="size-3.5" />
            Diff
          </TabsTrigger>
          <TabsTrigger
            value="review"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <MessageSquare className="size-3.5" />
            AI Review
          </TabsTrigger>
          <TabsTrigger
            value="server"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <Server className="size-3.5" />
            Dev Server
          </TabsTrigger>
          {currentSession && (
            <TabsTrigger
              value="session"
              className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
            >
              <Play className="size-3.5" />
              Session
            </TabsTrigger>
          )}
        </TabsList>

        <div className="flex-1 overflow-auto">
          <TabsContent value="plan" className="m-0 h-full">
            <PlanTab content={mockPlan} />
          </TabsContent>
          <TabsContent value="diff" className="m-0 h-full">
            <DiffTab diff={mockDiff} />
          </TabsContent>
          <TabsContent value="review" className="m-0 h-full">
            <ReviewTab content={mockReview} />
          </TabsContent>
          <TabsContent value="server" className="m-0 h-full">
            <DevServerTab taskId={task.id} />
          </TabsContent>
          {currentSession && (
            <TabsContent value="session" className="m-0 h-full">
              <SessionTab session={currentSession} />
            </TabsContent>
          )}
        </div>
      </Tabs>

      {/* Actions */}
      {(canApprove || canReject) && (
        <div className="border-t border-border p-4 flex gap-2">
          <Button variant="outline" className="flex-1 gap-2 bg-transparent" onClick={handleReject}>
            <RotateCcw className="size-4" />
            Reject
          </Button>
          <Button className="flex-1 gap-2" onClick={handleApprove}>
            <Check className="size-4" />
            Approve
          </Button>
        </div>
      )}
    </div>
  )
}
