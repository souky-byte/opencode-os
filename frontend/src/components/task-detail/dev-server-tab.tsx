"use client"

import { useState } from "react"
import { Button } from "~/components/ui/button"
import { ExternalLink, Play, RefreshCw, Square, Terminal } from "lucide-react"

interface DevServerTabProps {
  taskId: string
}

export function DevServerTab({ taskId }: DevServerTabProps) {
  const [isRunning, setIsRunning] = useState(true)
  const port = 3042

  return (
    <div className="p-4 space-y-4">
      <div className="rounded-lg border border-border p-4">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <div className={`size-2 rounded-full ${isRunning ? "bg-primary animate-pulse" : "bg-muted-foreground"}`} />
            <span className="text-sm font-medium">{isRunning ? "Running" : "Stopped"}</span>
          </div>
          <span className="text-xs text-muted-foreground font-mono">Port {port}</span>
        </div>

        <div className="flex gap-2">
          {isRunning ? (
            <>
              <Button
                variant="outline"
                size="sm"
                className="gap-1.5 bg-transparent"
                onClick={() => setIsRunning(false)}
              >
                <Square className="size-3" />
                Stop
              </Button>
              <Button variant="outline" size="sm" className="gap-1.5 bg-transparent">
                <RefreshCw className="size-3" />
                Restart
              </Button>
              <Button size="sm" className="gap-1.5 ml-auto" asChild>
                <a href={`http://localhost:${port}`} target="_blank" rel="noopener noreferrer">
                  <ExternalLink className="size-3" />
                  Open
                </a>
              </Button>
            </>
          ) : (
            <Button size="sm" className="gap-1.5" onClick={() => setIsRunning(true)}>
              <Play className="size-3" />
              Start Server
            </Button>
          )}
        </div>
      </div>

      <div className="rounded-lg border border-border overflow-hidden">
        <div className="flex items-center gap-2 bg-secondary px-3 py-2 border-b border-border">
          <Terminal className="size-3.5 text-muted-foreground" />
          <span className="text-xs font-medium">Server Logs</span>
        </div>
        <div className="bg-background p-3 font-mono text-xs text-muted-foreground max-h-64 overflow-auto">
          <div className="text-primary">$ npm run dev</div>
          <div className="mt-1">{">"} my-app@0.1.0 dev</div>
          <div>{">"} next dev</div>
          <div className="mt-2 text-foreground">▲ Next.js 15.1.0</div>
          <div className="text-foreground">- Local: http://localhost:{port}</div>
          <div className="mt-2 text-primary">✓ Ready in 1.2s</div>
          <div className="mt-1 text-muted-foreground">○ Compiling /page ...</div>
          <div className="text-primary">✓ Compiled /page in 420ms</div>
        </div>
      </div>
    </div>
  )
}
