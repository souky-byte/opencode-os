import NiceModal, { useModal } from "@ebay/nice-modal-react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState, useEffect } from "react";
import type { Task } from "@/api/generated/model";
import { getListTasksQueryKey } from "@/api/generated/tasks/tasks";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Loader } from "@/components/ui/loader";
import { defineModal } from "@/lib/modals";
import { customFetch } from "@/lib/api-fetcher";
import { DiffSummaryCard } from "@/components/complete/DiffSummaryCard";
import {
  ActionSelector,
  type CompleteAction,
} from "@/components/complete/ActionSelector";
import { PrOptionsForm } from "@/components/complete/PrOptionsForm";
import { CompletionSuccess } from "@/components/complete/CompletionSuccess";
import { cn } from "@/lib/utils";

// Types matching backend
interface DiffSummary {
  files_changed: number;
  additions: number;
  deletions: number;
}

interface CompletePreviewResponse {
  task_id: string;
  branch_name: string;
  base_branch: string;
  diff_summary: DiffSummary;
  suggested_pr_title: string;
  suggested_pr_body: string;
  github_available: boolean;
  has_uncommitted_changes: boolean;
}

interface PrInfo {
  number: number;
  url: string;
  title: string;
}

interface CompleteTaskResponse {
  success: boolean;
  pr: PrInfo | null;
  merge_result: {
    status: string;
    commit_sha?: string;
    files?: string[];
  } | null;
  worktree_cleaned: boolean;
}

interface CompleteTaskDialogProps {
  task: Task;
}

interface CompleteTaskResult {
  success: boolean;
  prUrl?: string;
}

type DialogState = "form" | "loading" | "success" | "error";

const CompleteTaskDialogComponent = NiceModal.create<CompleteTaskDialogProps>(
  ({ task }) => {
    const modal = useModal();
    const queryClient = useQueryClient();

    // Dialog state
    const [dialogState, setDialogState] = useState<DialogState>("form");
    const [error, setError] = useState<string | null>(null);
    const [completionResult, setCompletionResult] =
      useState<CompleteTaskResponse | null>(null);

    // Form state
    const [action, setAction] = useState<CompleteAction>("create_pr");
    const [prTitle, setPrTitle] = useState("");
    const [prBody, setPrBody] = useState("");
    const [baseBranch, setBaseBranch] = useState("main");
    const [isDraft, setIsDraft] = useState(false);
    const [cleanupWorktree, setCleanupWorktree] = useState(true);

    // User mode - default to developer for now
    const [userMode] = useState<"developer" | "basic">("developer");

    // Fetch preview data
    const {
      data: preview,
      isLoading: previewLoading,
      error: previewError,
    } = useQuery({
      queryKey: ["complete-preview", task.id],
      queryFn: async () => {
        const response = await customFetch<{ data: CompletePreviewResponse }>(
          `/api/tasks/${task.id}/complete/preview`,
        );
        return response.data;
      },
      staleTime: 30000,
    });

    // Initialize form with preview data
    useEffect(() => {
      if (preview) {
        setPrTitle(preview.suggested_pr_title);
        setPrBody(preview.suggested_pr_body);
        setBaseBranch(preview.base_branch);
        // If GitHub not available, default to merge_local
        if (!preview.github_available && action === "create_pr") {
          setAction("merge_local");
        }
      }
    }, [preview, action]);

    // Complete mutation
    const completeMutation = useMutation({
      mutationFn: async () => {
        const response = await customFetch<{ data: CompleteTaskResponse }>(
          `/api/tasks/${task.id}/complete`,
          {
            method: "POST",
            body: JSON.stringify({
              action,
              pr_options:
                action === "create_pr"
                  ? {
                      title: prTitle,
                      body: prBody,
                      base_branch: baseBranch,
                      draft: isDraft,
                    }
                  : undefined,
              merge_options:
                action === "merge_local"
                  ? {
                      commit_message: `Merge task: ${task.title}`,
                    }
                  : undefined,
              cleanup_worktree: cleanupWorktree,
            }),
          },
        );
        return response.data;
      },
      onSuccess: (data) => {
        setCompletionResult(data);
        setDialogState("success");
        void queryClient.invalidateQueries({
          queryKey: getListTasksQueryKey(),
        });
      },
      onError: (err: Error) => {
        setError(err.message || "Failed to complete task");
        setDialogState("error");
      },
    });

    const handleComplete = () => {
      setDialogState("loading");
      setError(null);
      completeMutation.mutate();
    };

    const handleClose = () => {
      if (completionResult?.success) {
        modal.resolve({ success: true, prUrl: completionResult.pr?.url });
      }
      void modal.hide();
    };

    const handleViewPr = () => {
      if (completionResult?.pr?.url) {
        window.open(completionResult.pr.url, "_blank");
      }
    };

    const getActionButtonText = () => {
      switch (action) {
        case "create_pr":
          return isDraft ? "Create Draft PR" : "Create PR & Complete";
        case "merge_local":
          return "Merge & Complete";
        case "complete_only":
          return "Complete Task";
      }
    };

    const renderContent = () => {
      if (previewLoading) {
        return (
          <div className="flex items-center justify-center py-12">
            <Loader size="default" message="Loading..." />
          </div>
        );
      }

      if (previewError) {
        return (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <div className="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
              <svg
                className="w-6 h-6 text-red-500"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="12" y1="8" x2="12" y2="12" />
                <line x1="12" y1="16" x2="12.01" y2="16" />
              </svg>
            </div>
            <p className="text-sm text-muted-foreground">
              Failed to load completion preview
            </p>
            <Button
              variant="outline"
              size="sm"
              className="mt-4"
              onClick={() => modal.hide()}
            >
              Close
            </Button>
          </div>
        );
      }

      if (dialogState === "success" && completionResult) {
        return (
          <CompletionSuccess
            pr={completionResult.pr}
            worktreeCleaned={completionResult.worktree_cleaned}
            onClose={handleClose}
            onViewPr={completionResult.pr ? handleViewPr : undefined}
          />
        );
      }

      if (dialogState === "loading") {
        return (
          <div className="flex flex-col items-center justify-center py-12">
            <Loader size="default" />
            <p className="mt-4 text-sm text-muted-foreground">
              {action === "create_pr"
                ? "Creating pull request..."
                : action === "merge_local"
                  ? "Merging changes..."
                  : "Completing task..."}
            </p>
          </div>
        );
      }

      if (dialogState === "error") {
        return (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <div className="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center mb-4">
              <svg
                className="w-6 h-6 text-red-500"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
            </div>
            <p className="text-sm font-medium text-foreground mb-1">
              Failed to complete task
            </p>
            <p className="text-sm text-muted-foreground mb-4">{error}</p>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDialogState("form")}
              >
                Try Again
              </Button>
              <Button size="sm" onClick={() => modal.hide()}>
                Close
              </Button>
            </div>
          </div>
        );
      }

      // Form state
      return (
        <>
          <div className="space-y-6">
            {/* Diff Summary */}
            {preview && (
              <DiffSummaryCard
                branchName={preview.branch_name}
                baseBranch={preview.base_branch}
                filesChanged={preview.diff_summary.files_changed}
                additions={preview.diff_summary.additions}
                deletions={preview.diff_summary.deletions}
              />
            )}

            {/* Action Selection */}
            <ActionSelector
              value={action}
              onChange={setAction}
              githubAvailable={preview?.github_available ?? false}
              mode={userMode}
            />

            {/* PR Options (shown when create_pr selected) */}
            {action === "create_pr" && (
              <PrOptionsForm
                title={prTitle}
                onTitleChange={setPrTitle}
                body={prBody}
                onBodyChange={setPrBody}
                baseBranch={baseBranch}
                onBaseBranchChange={setBaseBranch}
                isDraft={isDraft}
                onIsDraftChange={setIsDraft}
                mode={userMode}
              />
            )}

            {/* Cleanup toggle (developer mode only) */}
            {userMode === "developer" && action !== "complete_only" && (
              <label className="flex items-center gap-3 p-3 rounded-lg border border-border/50 bg-card/30 cursor-pointer hover:bg-card/50 transition-colors">
                <input
                  type="checkbox"
                  checked={cleanupWorktree}
                  onChange={(e) => setCleanupWorktree(e.target.checked)}
                  className="w-4 h-4 rounded border-muted-foreground/30 text-primary focus:ring-primary/50"
                />
                <div>
                  <div className="text-sm font-medium text-foreground">
                    Delete worktree after completion
                  </div>
                  <div className="text-xs text-muted-foreground/70">
                    Recommended - keeps your workspace clean
                  </div>
                </div>
              </label>
            )}
          </div>

          <DialogFooter className="mt-6">
            <Button variant="outline" onClick={() => modal.hide()}>
              Cancel
            </Button>
            <Button
              onClick={handleComplete}
              disabled={action === "create_pr" && !prTitle.trim()}
              className={cn(
                action === "create_pr" &&
                  "bg-[#238636] hover:bg-[#2ea043] text-white",
              )}
            >
              {getActionButtonText()}
            </Button>
          </DialogFooter>
        </>
      );
    };

    return (
      <Dialog
        open={modal.visible}
        onOpenChange={(open) => {
          if (!open && dialogState !== "loading") {
            handleClose();
          }
        }}
      >
        <DialogContent
          className={cn(
            "sm:max-w-lg",
            dialogState === "success" && "sm:max-w-md",
          )}
        >
          {dialogState !== "success" && (
            <DialogHeader>
              <DialogTitle>
                {userMode === "basic"
                  ? "Complete Task"
                  : `Complete: ${task.title}`}
              </DialogTitle>
              {userMode === "developer" && (
                <DialogDescription>
                  Choose how to save your changes and complete this task.
                </DialogDescription>
              )}
            </DialogHeader>
          )}

          {renderContent()}
        </DialogContent>
      </Dialog>
    );
  },
);

export const CompleteTaskDialog = defineModal<
  CompleteTaskDialogProps,
  CompleteTaskResult
>(CompleteTaskDialogComponent);
