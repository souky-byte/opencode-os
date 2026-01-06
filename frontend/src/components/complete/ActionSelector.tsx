import { cn } from "@/lib/utils";

export type CompleteAction = "create_pr" | "merge_local" | "complete_only";

interface ActionOption {
  value: CompleteAction;
  label: string;
  description: string;
  icon: React.ReactNode;
  disabled?: boolean;
  disabledReason?: string;
}

interface ActionSelectorProps {
  value: CompleteAction;
  onChange: (value: CompleteAction) => void;
  githubAvailable: boolean;
  mode: "developer" | "basic";
}

const GitHubIcon = () => (
  <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
    <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
  </svg>
);

const GitIcon = () => (
  <svg
    className="w-5 h-5"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
  >
    <circle cx="18" cy="18" r="3" />
    <circle cx="6" cy="6" r="3" />
    <path d="M13 6h3a2 2 0 0 1 2 2v7" />
    <path d="M6 9v12" />
  </svg>
);

const CheckIcon = () => (
  <svg
    className="w-5 h-5"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
  >
    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
    <polyline points="22 4 12 14.01 9 11.01" />
  </svg>
);

export function ActionSelector({
  value,
  onChange,
  githubAvailable,
  mode,
}: ActionSelectorProps) {
  const options: ActionOption[] = [
    {
      value: "create_pr",
      label: "Create Pull Request",
      description: "Push to GitHub and create a PR for code review",
      icon: <GitHubIcon />,
      disabled: !githubAvailable,
      disabledReason: "GitHub token not configured",
    },
    {
      value: "merge_local",
      label: "Merge to Main",
      description: "Apply changes directly to your main branch",
      icon: <GitIcon />,
    },
    ...(mode === "developer"
      ? [
          {
            value: "complete_only" as CompleteAction,
            label: "Just Complete",
            description: "Mark as done without merging (keep worktree)",
            icon: <CheckIcon />,
          },
        ]
      : []),
  ];

  if (mode === "basic") {
    // Basic mode: card-style buttons
    return (
      <div className="space-y-3">
        {options.map((option) => (
          <button
            key={option.value}
            type="button"
            onClick={() => !option.disabled && onChange(option.value)}
            disabled={option.disabled}
            className={cn(
              "w-full flex items-center gap-4 p-4 rounded-lg border transition-all text-left",
              option.disabled
                ? "opacity-50 cursor-not-allowed border-border/30 bg-muted/20"
                : value === option.value
                  ? "border-primary/50 bg-primary/5"
                  : "border-border/50 bg-card/30 hover:bg-card/50 hover:border-border"
            )}
          >
            <div
              className={cn(
                "w-10 h-10 rounded-lg flex items-center justify-center shrink-0",
                option.disabled
                  ? "bg-muted/30 text-muted-foreground/50"
                  : value === option.value
                    ? "bg-primary/10 text-primary"
                    : "bg-muted/50 text-muted-foreground"
              )}
            >
              {option.icon}
            </div>
            <div className="flex-1 min-w-0">
              <div
                className={cn(
                  "font-medium",
                  option.disabled
                    ? "text-muted-foreground/50"
                    : "text-foreground"
                )}
              >
                {option.label}
              </div>
              <div className="text-sm text-muted-foreground/70">
                {option.disabled ? option.disabledReason : option.description}
              </div>
            </div>
            {!option.disabled && (
              <svg
                className={cn(
                  "w-5 h-5 shrink-0 transition-colors",
                  value === option.value
                    ? "text-primary"
                    : "text-muted-foreground/30"
                )}
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <polyline points="9 18 15 12 9 6" />
              </svg>
            )}
          </button>
        ))}
      </div>
    );
  }

  // Developer mode: radio buttons
  return (
    <div className="space-y-2">
      <div className="text-sm font-medium text-foreground">
        What would you like to do?
      </div>
      <div className="space-y-1">
        {options.map((option) => (
          <label
            key={option.value}
            className={cn(
              "flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all",
              option.disabled
                ? "opacity-50 cursor-not-allowed border-border/30"
                : value === option.value
                  ? "border-primary/50 bg-primary/5"
                  : "border-transparent hover:bg-muted/30"
            )}
          >
            <input
              type="radio"
              name="complete-action"
              value={option.value}
              checked={value === option.value}
              onChange={() => onChange(option.value)}
              disabled={option.disabled}
              className="w-4 h-4 text-primary border-muted-foreground/30 focus:ring-primary/50"
            />
            <div
              className={cn(
                "w-8 h-8 rounded-md flex items-center justify-center shrink-0",
                option.disabled
                  ? "bg-muted/30 text-muted-foreground/50"
                  : value === option.value
                    ? "bg-primary/10 text-primary"
                    : "bg-muted/50 text-muted-foreground"
              )}
            >
              {option.icon}
            </div>
            <div className="flex-1 min-w-0">
              <div
                className={cn(
                  "text-sm font-medium",
                  option.disabled
                    ? "text-muted-foreground/50"
                    : "text-foreground"
                )}
              >
                {option.label}
              </div>
              <div className="text-xs text-muted-foreground/70">
                {option.disabled ? option.disabledReason : option.description}
              </div>
            </div>
          </label>
        ))}
      </div>
    </div>
  );
}
