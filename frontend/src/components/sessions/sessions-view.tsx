"use client"

import { mockSessions, mockTasks } from "~/lib/mock-data"
import { Bot, Pause, StopCircle } from "lucide-react"
import { Button } from "~/components/ui/button"
import { cn } from "~/lib/utils"

export function SessionsView() {
  const activeSessions = mockSessions.filter((s) => s.status === "running")

  return (
    <div className="h-full p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Active Sessions</h1>
          <p className="text-sm text-muted-foreground">
            {activeSessions.length} running • Real-time AI agent monitoring
          </p>
        </div>
      </div>

      <div className="space-y-4">
        {mockSessions.map((session) => {
          const task = mockTasks.find((t) => t.id === session.taskId)

          return (
            <div key={session.id} className="rounded-lg border border-border bg-card overflow-hidden">
              <div className="flex items-center justify-between p-4 border-b border-border">
                <div className="flex items-center gap-3">
                  <div
                    className={cn(
                      "size-10 rounded-lg flex items-center justify-center",
                      session.status === "running" ? "bg-primary/20" : "bg-secondary",
                    )}
                  >
                    <Bot
                      className={cn("size-5", session.status === "running" ? "text-primary" : "text-muted-foreground")}
                    />
                  </div>
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{task?.title || session.taskId}</span>
                      <span
                        className={cn(
                          "text-xs px-2 py-0.5 rounded-full",
                          session.status === "running"
                            ? "bg-primary/20 text-primary"
                            : "bg-secondary text-muted-foreground",
                        )}
                      >
                        {session.status}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 text-xs text-muted-foreground mt-0.5">
                      <span className="capitalize">{session.phase}</span>
                      <span>•</span>
                      <span className="font-mono">{session.id}</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Button variant="ghost" size="icon" className="size-8">
                    <Pause className="size-4" />
                  </Button>
                  <Button variant="ghost" size="icon" className="size-8 text-destructive">
                    <StopCircle className="size-4" />
                  </Button>
                </div>
              </div>

              <div className="p-4 bg-secondary/30 max-h-48 overflow-auto">
                <div className="space-y-2">
                  {session.messages.slice(-4).map((message) => (
                    <div key={message.id} className="flex gap-2 text-sm">
                      <span className="text-xs text-muted-foreground font-mono shrink-0">
                        {new Date(message.timestamp).toLocaleTimeString()}
                      </span>
                      <span className={cn(message.role === "agent" ? "text-foreground" : "text-muted-foreground")}>
                        {message.content}
                      </span>
                    </div>
                  ))}
                  {session.status === "running" && (
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <span className="text-xs font-mono shrink-0">{new Date().toLocaleTimeString()}</span>
                      <div className="flex gap-1">
                        <span
                          className="size-1.5 rounded-full bg-primary animate-bounce"
                          style={{ animationDelay: "0ms" }}
                        />
                        <span
                          className="size-1.5 rounded-full bg-primary animate-bounce"
                          style={{ animationDelay: "150ms" }}
                        />
                        <span
                          className="size-1.5 rounded-full bg-primary animate-bounce"
                          style={{ animationDelay: "300ms" }}
                        />
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
