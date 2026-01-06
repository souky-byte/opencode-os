import { cn } from "@/lib/utils";

interface DiffSummaryCardProps {
  branchName: string;
  baseBranch: string;
  filesChanged: number;
  additions: number;
  deletions: number;
  className?: string;
}

export function DiffSummaryCard({
  branchName,
  baseBranch,
  filesChanged,
  additions,
  deletions,
  className,
}: DiffSummaryCardProps) {
  return (
    <div
      className={cn(
        "rounded-lg bg-card/50 border border-border/50 p-4",
        className
      )}
    >
      <div className="flex items-center justify-between">
        <div className="min-w-0">
          <div className="text-xs text-muted-foreground">Branch</div>
          <div className="font-mono text-sm text-foreground truncate">
            {branchName}
          </div>
        </div>
        <div className="flex items-center gap-2 text-muted-foreground/60">
          <svg
            className="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M5 12h14M12 5l7 7-7 7" />
          </svg>
        </div>
        <div className="min-w-0 text-right">
          <div className="text-xs text-muted-foreground">Base</div>
          <div className="font-mono text-sm text-foreground truncate">
            {baseBranch}
          </div>
        </div>
      </div>

      <div className="mt-4 flex items-center gap-4 text-sm">
        <div className="flex items-center gap-1.5">
          <span className="text-emerald-500 font-mono">+{additions}</span>
          <span className="text-muted-foreground/50">lines</span>
        </div>
        <div className="flex items-center gap-1.5">
          <span className="text-red-500 font-mono">-{deletions}</span>
          <span className="text-muted-foreground/50">lines</span>
        </div>
        <div className="flex-1" />
        <div className="flex items-center gap-1.5">
          <svg
            className="w-4 h-4 text-muted-foreground/50"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          <span className="text-muted-foreground">
            {filesChanged} {filesChanged === 1 ? "file" : "files"}
          </span>
        </div>
      </div>
    </div>
  );
}
