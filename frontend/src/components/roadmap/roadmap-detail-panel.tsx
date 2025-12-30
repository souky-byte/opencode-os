"use client"

import { useState } from "react"
import { Button } from "~/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs"
import { Badge } from "~/components/ui/badge"
import { ArrowRight, CheckCircle2, Circle, Clock, FileText, ListTodo, Settings2, User, X } from "lucide-react"
import type { RoadmapItem } from "~/lib/mock-data"
import { mockRoadmapDetails, mockTasks } from "~/lib/mock-data"
import { cn } from "~/lib/utils"
import { MarkdownRenderer } from "~/components/markdown-renderer"

interface RoadmapDetailPanelProps {
  item: RoadmapItem
  onClose: () => void
}

export function RoadmapDetailPanel({ item, onClose }: RoadmapDetailPanelProps) {
  const [activeTab, setActiveTab] = useState("overview")
  const details = mockRoadmapDetails[item.id]
  const linkedTasks = mockTasks.filter((t) => item.linkedTaskIds.includes(t.id))

  const statusConfig = {
    planned: { icon: Circle, color: "text-chart-2", bg: "bg-chart-2/20", label: "Planned" },
    in_development: { icon: Clock, color: "text-primary", bg: "bg-primary/20", label: "In Development" },
    completed: { icon: CheckCircle2, color: "text-primary", bg: "bg-primary/20", label: "Completed" },
  }

  const config = statusConfig[item.status]
  const StatusIcon = config.icon

  return (
    <div className="fixed right-0 top-14 bottom-0 w-[540px] border-l border-border bg-background flex flex-col z-50">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border p-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <Badge variant="secondary" className={cn("text-xs", config.bg, config.color)}>
              <StatusIcon className="size-3 mr-1" />
              {config.label}
            </Badge>
            <Badge variant="outline" className="text-xs">
              {item.quarter}
            </Badge>
          </div>
          <h2 className="text-lg font-semibold truncate">{item.title}</h2>
        </div>
        <Button variant="ghost" size="icon" onClick={onClose}>
          <X className="size-4" />
        </Button>
      </div>

      {/* JTBD Banner */}
      <div className="border-b border-border p-4 bg-secondary/30">
        <div className="text-xs font-medium text-muted-foreground mb-1 uppercase tracking-wide">Job to be Done</div>
        <p className="text-sm text-foreground italic leading-relaxed">"{item.jtbd}"</p>
      </div>

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col overflow-hidden">
        <TabsList className="w-full justify-start rounded-none border-b border-border bg-transparent px-4 h-auto py-0">
          <TabsTrigger
            value="overview"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <FileText className="size-3.5" />
            Overview
          </TabsTrigger>
          <TabsTrigger
            value="stories"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <User className="size-3.5" />
            User Stories
          </TabsTrigger>
          <TabsTrigger
            value="technical"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <Settings2 className="size-3.5" />
            Technical
          </TabsTrigger>
          <TabsTrigger
            value="tasks"
            className="gap-1.5 rounded-none border-b-2 border-transparent data-[state=active]:border-primary data-[state=active]:bg-transparent py-3"
          >
            <ListTodo className="size-3.5" />
            Tasks
            {linkedTasks.length > 0 && (
              <span className="ml-1 text-xs bg-primary/20 text-primary px-1.5 rounded-full">{linkedTasks.length}</span>
            )}
          </TabsTrigger>
        </TabsList>

        <div className="flex-1 overflow-auto">
          {/* Overview Tab */}
          <TabsContent value="overview" className="m-0 h-full">
            <div className="p-4">
              <div className="rounded-lg border border-border bg-secondary/30 p-4">
                {details?.description ? (
                  <MarkdownRenderer content={details.description} />
                ) : (
                  <p className="text-sm text-muted-foreground">No description available.</p>
                )}
              </div>

              {/* Acceptance Criteria */}
              <div className="mt-6">
                <h3 className="text-sm font-semibold mb-3 flex items-center gap-2">
                  <CheckCircle2 className="size-4 text-primary" />
                  Acceptance Criteria
                </h3>
                <div className="space-y-2">
                  {item.acceptanceCriteria.map((criteria, index) => (
                    <div key={index} className="flex items-start gap-3 p-3 rounded-lg border border-border bg-card">
                      <div className="size-5 rounded border border-muted-foreground/30 flex items-center justify-center shrink-0 mt-0.5">
                        <span className="text-xs text-muted-foreground">{index + 1}</span>
                      </div>
                      <span className="text-sm text-foreground">{criteria}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </TabsContent>

          {/* User Stories Tab */}
          <TabsContent value="stories" className="m-0 h-full">
            <div className="p-4">
              <div className="space-y-3">
                {details?.userStories?.map((story, index) => (
                  <div key={index} className="p-4 rounded-lg border border-border bg-card">
                    <div className="flex items-start gap-3">
                      <div className="size-8 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                        <User className="size-4 text-primary" />
                      </div>
                      <div>
                        <div className="text-xs text-muted-foreground mb-1">User Story #{index + 1}</div>
                        <p className="text-sm text-foreground leading-relaxed">{story}</p>
                      </div>
                    </div>
                  </div>
                )) || <p className="text-sm text-muted-foreground">No user stories defined.</p>}
              </div>
            </div>
          </TabsContent>

          {/* Technical Tab */}
          <TabsContent value="technical" className="m-0 h-full">
            <div className="p-4">
              <div className="rounded-lg border border-border bg-secondary/30 p-4">
                {details?.technicalNotes ? (
                  <MarkdownRenderer content={details.technicalNotes} />
                ) : (
                  <p className="text-sm text-muted-foreground">No technical notes available.</p>
                )}
              </div>
            </div>
          </TabsContent>

          {/* Tasks Tab */}
          <TabsContent value="tasks" className="m-0 h-full">
            <div className="p-4">
              {linkedTasks.length > 0 ? (
                <div className="space-y-2">
                  {linkedTasks.map((task) => (
                    <div
                      key={task.id}
                      className="p-3 rounded-lg border border-border bg-card hover:border-primary/50 transition-colors cursor-pointer"
                    >
                      <div className="flex items-center justify-between mb-1">
                        <span className="text-xs font-mono text-muted-foreground">{task.id}</span>
                        <Badge
                          variant="outline"
                          className={cn(
                            "text-xs",
                            task.status === "DONE" && "border-primary text-primary",
                            task.status === "REVIEW" && "border-chart-5 text-chart-5",
                            ["PLANNING", "IN_PROGRESS", "AI_REVIEW"].includes(task.status) &&
                              "border-chart-1 text-chart-1",
                          )}
                        >
                          {task.status.replace("_", " ")}
                        </Badge>
                      </div>
                      <h4 className="font-medium text-sm">{task.title}</h4>
                      <p className="text-xs text-muted-foreground mt-1 line-clamp-1">{task.description}</p>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="text-center py-12">
                  <ListTodo className="size-12 text-muted-foreground/30 mx-auto mb-3" />
                  <p className="text-sm text-muted-foreground mb-4">No tasks linked yet</p>
                  <Button className="gap-2">
                    <ArrowRight className="size-4" />
                    Create Task from Roadmap
                  </Button>
                </div>
              )}
            </div>
          </TabsContent>
        </div>
      </Tabs>

      {/* Footer Actions */}
      <div className="border-t border-border p-4 flex gap-2">
        <Button variant="outline" className="flex-1 gap-2 bg-transparent">
          Edit Item
        </Button>
        <Button className="flex-1 gap-2">
          <ArrowRight className="size-4" />
          Move to Kanban
        </Button>
      </div>
    </div>
  )
}
