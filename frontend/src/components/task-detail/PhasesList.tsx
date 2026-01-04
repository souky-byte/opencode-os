import { useState } from "react";
import type { PhaseInfo, PhaseStatus } from "@/api/generated/model";
import { useGetTaskPhases } from "@/api/generated/phases/phases";
import { ActivityFeed } from "@/components/activity/ActivityFeed";
import { Loader } from "@/components/ui/loader";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useSessionActivitySSE } from "@/hooks/useSessionActivitySSE";
import { cn } from "@/lib/utils";
import {
  Check,
  Circle,
  Loader2,
  ChevronDown,
  ChevronRight,
} from "lucide-react";

interface PhasesListProps {
  taskId: string;
  className?: string;
}

const PHASE_STATUS_CONFIG: Record<
  PhaseStatus,
  {
    icon: React.ElementType;
    iconClass: string;
    lineClass: string;
  }
> = {
  pending: {
    icon: Circle,
    iconClass: "text-muted-foreground/40",
    lineClass: "bg-border/50",
  },
  running: {
    icon: Loader2,
    iconClass: "text-primary/70 animate-spin",
    lineClass: "bg-primary/30",
  },
  completed: {
    icon: Check,
    iconClass: "text-emerald-500/70",
    lineClass: "bg-emerald-500/30",
  },
};

function PhaseItem({
  phase,
  isLast,
  isExpanded,
  onToggle,
}: {
  phase: PhaseInfo;
  isLast: boolean;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  const config = PHASE_STATUS_CONFIG[phase.status];
  const Icon = config.icon;
  const hasSession = !!phase.session_id;

  const { activities, isConnected } = useSessionActivitySSE(
    isExpanded && hasSession ? (phase.session_id ?? null) : null,
    { enabled: isExpanded && hasSession },
  );

  return (
    <div className="relative flex gap-3">
      {/* Timeline column */}
      <div className="flex flex-col items-center">
        {/* Icon */}
        <div className="w-5 h-5 flex items-center justify-center shrink-0">
          <Icon className={cn("w-4 h-4", config.iconClass)} />
        </div>
        {/* Connecting line */}
        {!isLast && (
          <div className={cn("w-px flex-1 mt-1", config.lineClass)} />
        )}
      </div>

      {/* Content column */}
      <div className="flex-1 min-w-0 pb-4">
        <button
          type="button"
          onClick={onToggle}
          className="w-full flex items-start gap-2 text-left group"
        >
          <div className="flex-1 min-w-0">
            {/* Phase title - compact */}
            <div className="flex items-center gap-2">
              <span
                className={cn(
                  "text-sm font-medium",
                  phase.status === "completed" && "text-muted-foreground",
                  phase.status === "running" && "text-foreground",
                  phase.status === "pending" && "text-muted-foreground/60",
                )}
              >
                {phase.number}. {phase.title}
              </span>
            </div>

            {/* Summary - only for completed */}
            {phase.summary && (
              <p className="mt-0.5 text-xs text-muted-foreground/60 line-clamp-1">
                {phase.summary.summary}
              </p>
            )}

            {/* Changed files - compact pills */}
            {phase.summary?.files_changed &&
              phase.summary.files_changed.length > 0 && (
                <div className="mt-1 flex flex-wrap gap-1">
                  {phase.summary.files_changed
                    .slice(0, 2)
                    .map((file: string) => (
                      <span
                        key={file}
                        className="text-[10px] text-muted-foreground/50 bg-muted/50 px-1.5 py-0.5 rounded font-mono"
                      >
                        {file.split("/").pop()}
                      </span>
                    ))}
                  {phase.summary.files_changed.length > 2 && (
                    <span className="text-[10px] text-muted-foreground/40">
                      +{phase.summary.files_changed.length - 2}
                    </span>
                  )}
                </div>
              )}
          </div>

          {/* Expand indicator */}
          {hasSession && (
            <div className="text-muted-foreground/40 group-hover:text-muted-foreground/60 transition-colors">
              {isExpanded ? (
                <ChevronDown className="w-3.5 h-3.5" />
              ) : (
                <ChevronRight className="w-3.5 h-3.5" />
              )}
            </div>
          )}
        </button>

        {/* Expanded activity feed */}
        {isExpanded && hasSession && (
          <div className="mt-2 border border-border/50 rounded-md overflow-hidden bg-card/30">
            <div className="px-2 py-1 text-[10px] text-muted-foreground/50 border-b border-border/30 flex items-center gap-2">
              <span className="font-mono">
                {phase.session_id?.slice(0, 8)}…
              </span>
              {isConnected && phase.status === "running" && (
                <span className="w-1 h-1 rounded-full bg-emerald-500/60 animate-pulse" />
              )}
            </div>
            <ScrollArea className="h-[250px]">
              <ActivityFeed
                activities={activities}
                isConnected={isConnected}
                className="p-2"
              />
            </ScrollArea>
          </div>
        )}
      </div>
    </div>
  );
}

export function PhasesList({ taskId, className }: PhasesListProps) {
  // -1 means "explicitly collapsed by user", null means "use default (current_phase)"
  const [expandedPhase, setExpandedPhase] = useState<number | null>(null);

  const {
    data: phasesData,
    isLoading,
    error,
  } = useGetTaskPhases(taskId, {
    query: {
      staleTime: 10000,
      refetchInterval: (query) => {
        const data = query.state.data;
        if (data?.status === 200 && data.data.current_phase !== null) {
          return 5000;
        }
        return 30000;
      },
    },
  });

  if (isLoading) {
    return (
      <div className={cn("flex items-center justify-center py-6", className)}>
        <Loader size="sm" />
      </div>
    );
  }

  if (error) {
    return (
      <div className={cn("py-4 text-xs text-muted-foreground/60", className)}>
        Failed to load phases
      </div>
    );
  }

  const phases = phasesData?.status === 200 ? phasesData.data : null;

  if (!phases || phases.phases.length === 0) {
    return (
      <div className={cn("py-4 text-xs text-muted-foreground/60", className)}>
        No phases
      </div>
    );
  }

  // -1 means user explicitly collapsed all phases
  // null means use current_phase as default
  // positive number means user explicitly expanded that phase
  const effectiveExpanded =
    expandedPhase === -1
      ? null
      : expandedPhase !== null
        ? expandedPhase
        : phases.current_phase;

  return (
    <div className={cn(className)}>
      {/* Header - minimal */}
      <div className="flex items-center justify-between mb-3">
        <span className="text-xs font-medium text-muted-foreground/60 uppercase tracking-wider">
          Phases
        </span>
        <span className="text-[10px] text-muted-foreground/40 tabular-nums">
          {phases.phases.filter((p) => p.status === "completed").length}/
          {phases.total_phases}
        </span>
      </div>

      {/* Timeline */}
      <div>
        {phases.phases.map((phase: PhaseInfo, index: number) => (
          <PhaseItem
            key={phase.number}
            phase={phase}
            isLast={index === phases.phases.length - 1}
            isExpanded={effectiveExpanded === phase.number}
            onToggle={() => {
              if (effectiveExpanded === phase.number) {
                // Collapsing - use -1 to indicate explicit collapse
                setExpandedPhase(-1);
              } else {
                // Expanding a specific phase
                setExpandedPhase(phase.number);
              }
            }}
          />
        ))}
      </div>
    </div>
  );
}
