import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

interface PrOptionsFormProps {
  title: string;
  onTitleChange: (value: string) => void;
  body: string;
  onBodyChange: (value: string) => void;
  baseBranch: string;
  onBaseBranchChange: (value: string) => void;
  isDraft: boolean;
  onIsDraftChange: (value: boolean) => void;
  availableBranches?: string[];
  mode: "developer" | "basic";
}

export function PrOptionsForm({
  title,
  onTitleChange,
  body,
  onBodyChange,
  baseBranch,
  onBaseBranchChange,
  isDraft,
  onIsDraftChange,
  availableBranches = ["main", "master", "develop"],
  mode,
}: PrOptionsFormProps) {
  if (mode === "basic") {
    // Basic mode: minimal form, just title preview
    return (
      <div className="space-y-3 p-4 rounded-lg border border-border/50 bg-card/30">
        <div className="text-sm text-muted-foreground">
          PR will be created with title:
        </div>
        <div className="font-medium text-foreground">{title}</div>
      </div>
    );
  }

  // Developer mode: full form
  return (
    <div className="space-y-4 p-4 rounded-lg border border-border/50 bg-card/30">
      <div className="space-y-2">
        <label htmlFor="pr-title" className="text-sm font-medium text-foreground">
          Title
        </label>
        <Input
          id="pr-title"
          value={title}
          onChange={(e) => onTitleChange(e.target.value)}
          placeholder="Pull request title"
        />
      </div>

      <div className="space-y-2">
        <label htmlFor="pr-body" className="text-sm font-medium text-foreground">
          Description
        </label>
        <Textarea
          id="pr-body"
          value={body}
          onChange={(e) => onBodyChange(e.target.value)}
          placeholder="Describe your changes..."
          rows={4}
          className="resize-none"
        />
      </div>

      <div className="flex items-center gap-4">
        <div className="flex-1 space-y-2">
          <label
            htmlFor="base-branch"
            className="text-sm font-medium text-foreground"
          >
            Base Branch
          </label>
          <select
            id="base-branch"
            value={baseBranch}
            onChange={(e) => onBaseBranchChange(e.target.value)}
            className={cn(
              "w-full h-9 px-3 rounded-md border border-input bg-background text-sm",
              "focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
            )}
          >
            {availableBranches.map((branch) => (
              <option key={branch} value={branch}>
                {branch}
              </option>
            ))}
          </select>
        </div>

        <div className="flex-1">
          <label className="flex items-center gap-3 cursor-pointer p-3 rounded-lg hover:bg-muted/30 transition-colors">
            <input
              type="checkbox"
              checked={isDraft}
              onChange={(e) => onIsDraftChange(e.target.checked)}
              className="w-4 h-4 rounded border-muted-foreground/30 text-primary focus:ring-primary/50"
            />
            <div>
              <div className="text-sm font-medium text-foreground">
                Draft PR
              </div>
              <div className="text-xs text-muted-foreground/70">
                Not ready for review yet
              </div>
            </div>
          </label>
        </div>
      </div>
    </div>
  );
}
