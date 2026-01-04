import { useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { ReviewFinding } from "@/api/generated/model/reviewFinding";
import { FindingStatus } from "@/api/generated/model/findingStatus";
import {
  useFixFindings,
  useSkipFindings,
  getGetTaskFindingsQueryKey,
  getListTasksQueryKey,
} from "@/api/generated/tasks/tasks";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Collapsible } from "@/components/ui/collapsible";
import { Icon } from "@/components/ui/icon";
import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "@/stores/useToastStore";
import { cn } from "@/lib/utils";

interface ProblemsTabProps {
  taskId: string;
  findings: ReviewFinding[];
  summary: string;
}

const SEVERITY_CONFIG = {
  error: {
    label: "Errors",
    color: "text-red-500",
    bg: "bg-red-500/10",
    border: "border-red-500/30",
    icon: "alert-circle" as const,
  },
  warning: {
    label: "Warnings",
    color: "text-yellow-500",
    bg: "bg-yellow-500/10",
    border: "border-yellow-500/30",
    icon: "alert-triangle" as const,
  },
  info: {
    label: "Info",
    color: "text-blue-500",
    bg: "bg-blue-500/10",
    border: "border-blue-500/30",
    icon: "info" as const,
  },
} as const;

type SeverityKey = keyof typeof SEVERITY_CONFIG;

export function ProblemsTab({ taskId, findings, summary }: ProblemsTabProps) {
  const queryClient = useQueryClient();
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());

  // Separate pending and fixed findings
  const { pendingFindings, fixedFindings } = useMemo(() => {
    const pending: ReviewFinding[] = [];
    const fixed: ReviewFinding[] = [];
    for (const finding of findings) {
      if (finding.status === FindingStatus.fixed) {
        fixed.push(finding);
      } else if (finding.status !== FindingStatus.skipped) {
        pending.push(finding);
      }
    }
    return { pendingFindings: pending, fixedFindings: fixed };
  }, [findings]);

  const groupedFindings = useMemo(() => {
    const groups: Record<SeverityKey, ReviewFinding[]> = {
      error: [],
      warning: [],
      info: [],
    };
    for (const finding of pendingFindings) {
      const severity = finding.severity as SeverityKey;
      if (severity in groups) {
        groups[severity].push(finding);
      }
    }
    return groups;
  }, [pendingFindings]);

  const fixFindings = useFixFindings({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListTasksQueryKey(),
        });
        void queryClient.invalidateQueries({
          queryKey: getGetTaskFindingsQueryKey(taskId),
        });
        setSelectedIds(new Set());
        toast.success("Fix started");
      },
      onError: () => {
        toast.error("Failed to start fix");
      },
    },
  });

  const skipFindings = useSkipFindings({
    mutation: {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getListTasksQueryKey(),
        });
        toast.success("Findings skipped, task moved to review");
      },
      onError: () => {
        toast.error("Failed to skip findings");
      },
    },
  });

  const toggleFinding = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const toggleSeverityGroup = (severity: SeverityKey) => {
    const groupFindings = groupedFindings[severity];
    const allSelected = groupFindings.every((f) => selectedIds.has(f.id));
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (allSelected) {
        for (const f of groupFindings) {
          next.delete(f.id);
        }
      } else {
        for (const f of groupFindings) {
          next.add(f.id);
        }
      }
      return next;
    });
  };

  const getGroupCheckState = (severity: SeverityKey) => {
    const group = groupedFindings[severity];
    if (group.length === 0) return { checked: false, indeterminate: false };
    const selectedCount = group.filter((f) => selectedIds.has(f.id)).length;
    if (selectedCount === 0) return { checked: false, indeterminate: false };
    if (selectedCount === group.length)
      return { checked: true, indeterminate: false };
    return { checked: false, indeterminate: true };
  };

  const handleFixSelected = () => {
    if (selectedIds.size === 0) return;
    fixFindings.mutate({
      id: taskId,
      data: { finding_ids: Array.from(selectedIds) },
    });
  };

  const handleFixAll = () => {
    fixFindings.mutate({
      id: taskId,
      data: { fix_all: true },
    });
  };

  const handleSkip = () => {
    skipFindings.mutate({ id: taskId });
  };

  const isLoading = fixFindings.isPending || skipFindings.isPending;

  // All findings are fixed
  if (pendingFindings.length === 0 && fixedFindings.length > 0) {
    return (
      <ScrollArea className="h-full">
        <div className="p-4 space-y-4">
          <div className="flex items-center gap-3 p-4 rounded-lg border border-green-500/30 bg-green-500/10">
            <Icon name="check-circle" size="md" className="text-green-500" />
            <div>
              <p className="font-medium text-green-500">All issues fixed!</p>
              <p className="text-sm text-muted-foreground">
                {fixedFindings.length} finding
                {fixedFindings.length !== 1 ? "s" : ""} have been resolved.
              </p>
            </div>
          </div>

          {/* Show fixed findings collapsed */}
          <Collapsible>
            <div className="rounded-lg border border-green-500/20 bg-green-500/5">
              <Collapsible.Trigger className="w-full flex items-center gap-2 p-3">
                <Icon
                  name="check-circle"
                  size="sm"
                  className="text-green-500"
                />
                <span className="font-medium text-sm text-green-500">
                  Fixed ({fixedFindings.length})
                </span>
                <Collapsible.Arrow />
              </Collapsible.Trigger>
              <Collapsible.Content>
                <div className="border-t border-border/50">
                  {fixedFindings.map((finding) => (
                    <FixedFindingItem key={finding.id} finding={finding} />
                  ))}
                </div>
              </Collapsible.Content>
            </div>
          </Collapsible>
        </div>
      </ScrollArea>
    );
  }

  return (
    <ScrollArea className="h-full">
      <div className="p-4 space-y-4">
        {summary && <p className="text-sm text-muted-foreground">{summary}</p>}

        {pendingFindings.length > 0 && (
          <div className="flex gap-2 sticky top-0 bg-background py-2 z-10">
            <Button
              size="sm"
              onClick={handleFixSelected}
              disabled={selectedIds.size === 0 || isLoading}
            >
              {fixFindings.isPending ? (
                <Icon
                  name="loading"
                  size="sm"
                  className="animate-spin mr-1.5"
                />
              ) : null}
              Fix selected ({selectedIds.size})
            </Button>
            <Button
              size="sm"
              variant="secondary"
              onClick={handleFixAll}
              disabled={isLoading}
            >
              Fix all ({pendingFindings.length})
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={handleSkip}
              disabled={isLoading}
            >
              Skip
            </Button>
          </div>
        )}

        <div className="space-y-3">
          {(Object.keys(SEVERITY_CONFIG) as SeverityKey[]).map((severity) => {
            const group = groupedFindings[severity];
            if (group.length === 0) return null;

            const config = SEVERITY_CONFIG[severity];
            const checkState = getGroupCheckState(severity);

            return (
              <SeverityGroup
                key={severity}
                severity={severity}
                config={config}
                findings={group}
                selectedIds={selectedIds}
                onToggleFinding={toggleFinding}
                groupCheckState={checkState}
                onToggleGroup={() => toggleSeverityGroup(severity)}
              />
            );
          })}
        </div>

        {/* Show fixed findings if any */}
        {fixedFindings.length > 0 && (
          <Collapsible>
            <div className="rounded-lg border border-green-500/20 bg-green-500/5">
              <Collapsible.Trigger className="w-full flex items-center gap-2 p-3">
                <Icon
                  name="check-circle"
                  size="sm"
                  className="text-green-500"
                />
                <span className="font-medium text-sm text-green-500">
                  Fixed ({fixedFindings.length})
                </span>
                <Collapsible.Arrow />
              </Collapsible.Trigger>
              <Collapsible.Content>
                <div className="border-t border-border/50">
                  {fixedFindings.map((finding) => (
                    <FixedFindingItem key={finding.id} finding={finding} />
                  ))}
                </div>
              </Collapsible.Content>
            </div>
          </Collapsible>
        )}
      </div>
    </ScrollArea>
  );
}

interface SeverityGroupProps {
  severity: SeverityKey;
  config: (typeof SEVERITY_CONFIG)[SeverityKey];
  findings: ReviewFinding[];
  selectedIds: Set<string>;
  onToggleFinding: (id: string) => void;
  groupCheckState: { checked: boolean; indeterminate: boolean };
  onToggleGroup: () => void;
}

function SeverityGroup({
  severity,
  config,
  findings,
  selectedIds,
  onToggleFinding,
  groupCheckState,
  onToggleGroup,
}: SeverityGroupProps) {
  return (
    <Collapsible defaultOpen={severity === "error"}>
      <div className={cn("rounded-lg border", config.border, config.bg)}>
        <div className="flex items-center gap-3 p-3">
          <Checkbox
            checked={groupCheckState.checked}
            indeterminate={groupCheckState.indeterminate}
            onCheckedChange={onToggleGroup}
          />
          <Collapsible.Trigger className="flex-1 flex items-center gap-2">
            <Icon name={config.icon} size="sm" className={config.color} />
            <span className={cn("font-medium text-sm", config.color)}>
              {config.label} ({findings.length})
            </span>
            <Collapsible.Arrow />
          </Collapsible.Trigger>
        </div>
        <Collapsible.Content>
          <div className="border-t border-border/50">
            {findings.map((finding) => (
              <FindingItem
                key={finding.id}
                finding={finding}
                isSelected={selectedIds.has(finding.id)}
                onToggle={() => onToggleFinding(finding.id)}
              />
            ))}
          </div>
        </Collapsible.Content>
      </div>
    </Collapsible>
  );
}

interface FindingItemProps {
  finding: ReviewFinding;
  isSelected: boolean;
  onToggle: () => void;
}

function FindingItem({ finding, isSelected, onToggle }: FindingItemProps) {
  return (
    <Collapsible>
      <div className="flex items-start gap-3 px-3 py-2 hover:bg-accent/30 transition-colors">
        <Checkbox
          checked={isSelected}
          onCheckedChange={onToggle}
          className="mt-0.5"
        />
        <Collapsible.Trigger className="flex-1 flex items-start gap-2 text-left">
          <div className="flex-1 min-w-0">
            <p className="font-medium text-sm">{finding.title}</p>
            {finding.file_path && (
              <p className="text-xs text-muted-foreground font-mono truncate">
                {finding.file_path}
                {finding.line_start != null && `:${finding.line_start}`}
                {finding.line_end != null &&
                  finding.line_end !== finding.line_start &&
                  `-${finding.line_end}`}
              </p>
            )}
          </div>
          <Collapsible.Arrow />
        </Collapsible.Trigger>
      </div>
      <Collapsible.Content className="px-3 pb-3 ml-7">
        <p className="text-sm text-muted-foreground whitespace-pre-wrap">
          {finding.description}
        </p>
      </Collapsible.Content>
    </Collapsible>
  );
}

interface FixedFindingItemProps {
  finding: ReviewFinding;
}

function FixedFindingItem({ finding }: FixedFindingItemProps) {
  return (
    <Collapsible>
      <div className="flex items-start gap-3 px-3 py-2 hover:bg-accent/30 transition-colors opacity-60">
        <Icon name="check" size="sm" className="text-green-500 mt-0.5" />
        <Collapsible.Trigger className="flex-1 flex items-start gap-2 text-left">
          <div className="flex-1 min-w-0">
            <p className="font-medium text-sm line-through text-muted-foreground">
              {finding.title}
            </p>
            {finding.file_path && (
              <p className="text-xs text-muted-foreground font-mono truncate">
                {finding.file_path}
                {finding.line_start != null && `:${finding.line_start}`}
                {finding.line_end != null &&
                  finding.line_end !== finding.line_start &&
                  `-${finding.line_end}`}
              </p>
            )}
          </div>
          <Collapsible.Arrow />
        </Collapsible.Trigger>
      </div>
      <Collapsible.Content className="px-3 pb-3 ml-7">
        <p className="text-sm text-muted-foreground whitespace-pre-wrap">
          {finding.description}
        </p>
      </Collapsible.Content>
    </Collapsible>
  );
}
