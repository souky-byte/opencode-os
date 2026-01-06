import { Button } from "@/components/ui/button";

interface PrInfo {
  number: number;
  url: string;
  title: string;
}

interface CompletionSuccessProps {
  pr?: PrInfo | null;
  worktreeCleaned: boolean;
  onClose: () => void;
  onViewPr?: () => void;
}

export function CompletionSuccess({
  pr,
  worktreeCleaned,
  onClose,
  onViewPr,
}: CompletionSuccessProps) {
  return (
    <div className="flex flex-col items-center py-6 px-4 text-center">
      {/* Success icon */}
      <div className="w-16 h-16 rounded-full bg-emerald-500/10 flex items-center justify-center mb-4">
        <svg
          className="w-8 h-8 text-emerald-500"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
      </div>

      {/* Title */}
      <h3 className="text-lg font-semibold text-foreground mb-2">
        {pr ? "Pull Request Created!" : "Task Completed!"}
      </h3>

      {/* PR info */}
      {pr && (
        <div className="w-full max-w-sm mb-4">
          <div className="p-3 rounded-lg border border-border/50 bg-card/30 text-left">
            <div className="flex items-center gap-2 mb-1">
              <svg
                className="w-4 h-4 text-muted-foreground"
                viewBox="0 0 24 24"
                fill="currentColor"
              >
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
              </svg>
              <span className="text-sm font-medium text-foreground">
                PR #{pr.number}
              </span>
            </div>
            <p className="text-sm text-muted-foreground truncate">{pr.title}</p>
            <a
              href={pr.url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-xs text-primary hover:underline truncate block mt-1"
            >
              {pr.url}
            </a>
          </div>
        </div>
      )}

      {/* Status list */}
      <div className="w-full max-w-sm mb-6">
        <div className="space-y-2 text-left text-sm">
          <div className="flex items-center gap-2 text-muted-foreground">
            <svg
              className="w-4 h-4 text-emerald-500"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <polyline points="20 6 9 17 4 12" />
            </svg>
            <span>Task marked as done</span>
          </div>
          {pr && (
            <div className="flex items-center gap-2 text-muted-foreground">
              <svg
                className="w-4 h-4 text-emerald-500"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <span>Changes pushed to GitHub</span>
            </div>
          )}
          {worktreeCleaned && (
            <div className="flex items-center gap-2 text-muted-foreground">
              <svg
                className="w-4 h-4 text-emerald-500"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <span>Worktree cleaned up</span>
            </div>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3">
        {pr && onViewPr && (
          <Button onClick={onViewPr} variant="outline">
            <svg
              className="w-4 h-4 mr-2"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
            View on GitHub
          </Button>
        )}
        <Button onClick={onClose}>Close</Button>
      </div>
    </div>
  );
}
