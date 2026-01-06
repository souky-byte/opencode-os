import type { PrReviewComment } from "@/api/generated/model";
import { useFixFromPrComments } from "@/api/generated/pull-requests/pull-requests";
import { useExecuteTask } from "@/api/generated/tasks/tasks";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Loader } from "@/components/ui/loader";
import { usePullRequestStore } from "@/stores/usePullRequestStore";
import { useSidebarStore } from "@/stores/useSidebarStore";
import { formatDistanceToNow } from "@/lib/date";
import { Sparkles, MessageSquare, FileCode, Loader2 } from "lucide-react";
import { useMemo } from "react";
import { useQueryClient } from "@tanstack/react-query";

interface PrCommentsSidebarProps {
  comments: PrReviewComment[];
  prNumber: number;
  isLoading: boolean;
}

// Group comments by thread (in_reply_to_id)
function groupCommentsByThread(comments: PrReviewComment[]) {
  const threads: Map<number, PrReviewComment[]> = new Map();
  const rootComments: PrReviewComment[] = [];

  // First pass: identify root comments
  for (const comment of comments) {
    if (!comment.in_reply_to_id) {
      rootComments.push(comment);
      threads.set(comment.id, [comment]);
    }
  }

  // Second pass: add replies to their threads
  for (const comment of comments) {
    if (comment.in_reply_to_id) {
      const thread = threads.get(comment.in_reply_to_id);
      if (thread) {
        thread.push(comment);
      }
    }
  }

  return rootComments.map((root) => ({
    root,
    replies: threads.get(root.id)?.slice(1) ?? [],
  }));
}

function PrCommentsSidebar({
  comments,
  prNumber,
  isLoading,
}: PrCommentsSidebarProps) {
  const {
    selectedCommentIds,
    toggleCommentSelection,
    selectAllComments,
    clearCommentSelection,
    selectPr,
  } = usePullRequestStore();
  const { setActiveView } = useSidebarStore();
  const queryClient = useQueryClient();

  const executeTask = useExecuteTask();
  const fixMutation = useFixFromPrComments();

  const threads = useMemo(() => groupCommentsByThread(comments), [comments]);

  const allRootIds = threads.map((t) => t.root.id);
  const allSelected =
    allRootIds.length > 0 &&
    allRootIds.every((id) => selectedCommentIds.includes(id));

  const handleSelectAll = () => {
    if (allSelected) {
      clearCommentSelection();
    } else {
      selectAllComments(allRootIds);
    }
  };

  const handleFixWithAi = () => {
    fixMutation.mutate(
      {
        number: prNumber,
        data: { comment_ids: selectedCommentIds },
      },
      {
        onSuccess: (response) => {
          if (response.status === 201) {
            const task = response.data.task;
            // Invalidate tasks to show the new task in kanban
            void queryClient.invalidateQueries({ queryKey: ["/api/tasks"] });
            // Auto-execute the task to start AI session
            executeTask.mutate({ id: task.id });
          }
          clearCommentSelection();
          selectPr(null);
          setActiveView("kanban");
        },
      },
    );
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader message="Loading comments..." />
      </div>
    );
  }

  if (threads.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-4">
        <div className="text-center text-muted-foreground">
          <MessageSquare className="w-8 h-8 mx-auto mb-2 opacity-50" />
          <p className="text-sm">No review comments</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full border-l">
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b">
        <div className="flex items-center gap-2">
          <MessageSquare className="w-4 h-4 text-muted-foreground" />
          <span className="text-sm font-medium">
            Comments ({threads.length})
          </span>
        </div>
        <Button
          variant="ghost"
          size="sm"
          className="text-xs h-6 px-2"
          onClick={handleSelectAll}
        >
          {allSelected ? "Deselect all" : "Select all"}
        </Button>
      </div>

      {/* Comments list */}
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-2">
          {threads.map(({ root, replies }) => (
            <div
              key={root.id}
              className="rounded-lg border bg-card p-3 space-y-2"
            >
              {/* File path & line */}
              <div className="flex items-start gap-2">
                <Checkbox
                  checked={selectedCommentIds.includes(root.id)}
                  onCheckedChange={() => toggleCommentSelection(root.id)}
                  className="mt-0.5"
                />
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1 text-xs text-muted-foreground">
                    <FileCode className="w-3 h-3" />
                    <span className="truncate font-mono">{root.path}</span>
                    {root.line && (
                      <span className="shrink-0">L{root.line}</span>
                    )}
                  </div>
                </div>
              </div>

              {/* Root comment */}
              <div className="ml-6">
                <div className="flex items-center gap-2 mb-1">
                  <img
                    src={root.user.avatar_url}
                    alt={root.user.login}
                    className="w-4 h-4 rounded-full"
                  />
                  <span className="text-xs font-medium">{root.user.login}</span>
                  <span className="text-[10px] text-muted-foreground">
                    {formatDistanceToNow(new Date(root.created_at))}
                  </span>
                </div>
                <p className="text-xs text-muted-foreground line-clamp-3">
                  {root.body}
                </p>
              </div>

              {/* Replies count */}
              {replies.length > 0 && (
                <div className="ml-6 text-[10px] text-muted-foreground">
                  {replies.length} {replies.length === 1 ? "reply" : "replies"}
                </div>
              )}
            </div>
          ))}
        </div>
      </ScrollArea>

      {/* Fix with AI button */}
      {selectedCommentIds.length > 0 && (
        <div className="p-3 border-t">
          <Button
            className="w-full gap-2"
            onClick={handleFixWithAi}
            variant="default"
            disabled={fixMutation.isPending}
          >
            {fixMutation.isPending ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                Creating task...
              </>
            ) : (
              <>
                <Sparkles className="w-4 h-4" />
                Fix {selectedCommentIds.length} comment
                {selectedCommentIds.length > 1 ? "s" : ""} with AI
              </>
            )}
          </Button>
        </div>
      )}
    </div>
  );
}

export { PrCommentsSidebar };
