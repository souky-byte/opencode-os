"use client"

import { Bot, Cpu } from "lucide-react"
import type { Session } from "~/lib/mock-data"
import { cn } from "~/lib/utils"

interface SessionTabProps {
  session: Session
}

export function SessionTab({ session }: SessionTabProps) {
  return (
    <div className="p-4">
      <div className="flex items-center gap-2 mb-4">
        <div
          className={cn(
            "size-2 rounded-full",
            session.status === "running" ? "bg-primary animate-pulse" : "bg-muted-foreground",
          )}
        />
        <span className="text-sm font-medium capitalize">{session.phase}</span>
        <span className="text-xs text-muted-foreground">•</span>
        <span className="text-xs text-muted-foreground font-mono">{session.id}</span>
      </div>

      <div className="space-y-3">
        {session.messages.map((message) => (
          <div key={message.id} className="flex gap-3">
            <div
              className={cn(
                "shrink-0 size-6 rounded-full flex items-center justify-center",
                message.role === "agent" ? "bg-primary/20" : "bg-secondary",
              )}
            >
              {message.role === "agent" ? (
                <Bot className="size-3 text-primary" />
              ) : (
                <Cpu className="size-3 text-muted-foreground" />
              )}
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-xs text-muted-foreground mb-0.5">
                {new Date(message.timestamp).toLocaleTimeString()}
              </div>
              <p className="text-sm">{message.content}</p>
            </div>
          </div>
        ))}
        {session.status === "running" && (
          <div className="flex gap-3 items-center">
            <div className="shrink-0 size-6 rounded-full bg-primary/20 flex items-center justify-center">
              <Bot className="size-3 text-primary animate-pulse" />
            </div>
            <div className="flex gap-1">
              <span className="size-1.5 rounded-full bg-primary animate-bounce" style={{ animationDelay: "0ms" }} />
              <span className="size-1.5 rounded-full bg-primary animate-bounce" style={{ animationDelay: "150ms" }} />
              <span className="size-1.5 rounded-full bg-primary animate-bounce" style={{ animationDelay: "300ms" }} />
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
