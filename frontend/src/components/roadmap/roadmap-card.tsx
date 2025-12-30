"use client"

import { Badge } from "~/components/ui/badge"
import { Button } from "~/components/ui/button"
import { ArrowRight, CheckCircle2, Circle, Clock } from "lucide-react"
import type { RoadmapItem } from "~/lib/mock-data"
import { cn } from "~/lib/utils"

interface RoadmapCardProps {
  item: RoadmapItem
  onClick: () => void
}

export function RoadmapCard({ item, onClick }: RoadmapCardProps) {
  const statusConfig = {
    planned: { icon: Circle, color: "text-chart-2", bg: "bg-chart-2/20", label: "Planned" },
    in_development: { icon: Clock, color: "text-primary", bg: "bg-primary/20", label: "In Development" },
    completed: { icon: CheckCircle2, color: "text-primary", bg: "bg-primary/20", label: "Completed" },
  }

  const config = statusConfig[item.status]
  const StatusIcon = config.icon

  return (
    <div
      className="rounded-lg border border-border bg-card p-4 hover:border-primary/50 transition-colors cursor-pointer"
      onClick={onClick}
    >
      <div className="flex items-start justify-between gap-2 mb-3">
        <h3 className="font-semibold leading-tight">{item.title}</h3>
        <Badge variant="secondary" className={cn("shrink-0", config.bg, config.color)}>
          <StatusIcon className="size-3 mr-1" />
          {config.label}
        </Badge>
      </div>

      <p className="text-sm text-muted-foreground mb-4 line-clamp-2">{item.jtbd}</p>

      <div className="mb-4">
        <div className="text-xs text-muted-foreground mb-2">Acceptance Criteria</div>
        <div className="space-y-1">
          {item.acceptanceCriteria.slice(0, 3).map((criteria, index) => (
            <div key={index} className="flex items-center gap-2 text-xs">
              <div className="size-1.5 rounded-full bg-muted-foreground" />
              <span className="text-muted-foreground">{criteria}</span>
            </div>
          ))}
          {item.acceptanceCriteria.length > 3 && (
            <div className="text-xs text-muted-foreground">+{item.acceptanceCriteria.length - 3} more</div>
          )}
        </div>
      </div>

      <div className="flex items-center justify-between">
        <div className="text-xs text-muted-foreground">
          {item.linkedTaskIds.length > 0 ? (
            <span>{item.linkedTaskIds.length} linked tasks</span>
          ) : (
            <span>No linked tasks</span>
          )}
        </div>
        <Button
          variant="ghost"
          size="sm"
          className="gap-1 text-xs h-7"
          onClick={(e) => {
            e.stopPropagation()
          }}
        >
          Move to Kanban
          <ArrowRight className="size-3" />
        </Button>
      </div>
    </div>
  )
}
