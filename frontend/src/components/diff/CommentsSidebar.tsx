import { useState } from "react";
import {
  useListComments,
  useDeleteComment,
  useSendCommentsToFix,
  getListCommentsQueryKey,
} from "@/api/generated/comments/comments";
import type { ReviewCommentResponse } from "@/api/generated/model";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { useQueryClient } from "@tanstack/react-query";

interface CommentsSidebarProps {
  taskId: string;
  onCommentClick?: (comment: ReviewCommentResponse) => void;
}

export function CommentsSidebar({ taskId, onCommentClick }: CommentsSidebarProps) {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const queryClient = useQueryClient();

  const { data: commentsData, isLoading } = useListComments(taskId, {
    query: { staleTime: 5000 },
  });

  const { mutate: deleteComment } = useDeleteComment({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListCommentsQueryKey(taskId) });
      },
    },
  });

  const { mutate: sendToFix, isPending: isSending } = useSendCommentsToFix({
    mutation: {
      onSuccess: () => {
        setSelectedIds(new Set());
        queryClient.invalidateQueries({ queryKey: getListCommentsQueryKey(taskId) });
      },
    },
  });

  const comments = commentsData?.data?.comments ?? [];

  const handleToggleSelect = (id: string) => {
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

  const handleSelectAll = () => {
    if (selectedIds.size === comments.length) {
      setSelectedIds(new Set());
    } else {
      setSelectedIds(new Set(comments.map((c) => c.id)));
    }
  };

  const handleDelete = (commentId: string) => {
    deleteComment({ taskId, commentId });
    setSelectedIds((prev) => {
      const next = new Set(prev);
      next.delete(commentId);
      return next;
    });
  };

  const handleSendToFix = () => {
    if (selectedIds.size === 0) return;
    sendToFix({
      taskId,
      data: { comment_ids: Array.from(selectedIds) },
    });
  };

  const formatLineRange = (start: number, end: number) => {
    return start === end ? `L${start}` : `L${Math.min(start, end)}-${Math.max(start, end)}`;
  };

  if (isLoading) {
    return (
      <div className="w-80 border-l border-white/[0.06] bg-[#0a0a0f] flex items-center justify-center">
        <div className="text-xs text-white/30">Loading comments...</div>
      </div>
    );
  }

  return (
    <aside className="w-80 border-l border-white/[0.06] bg-[#0a0a0f] flex flex-col">
      {/* Header */}
      <div className="px-3 py-2.5 border-b border-white/[0.04] flex items-center justify-between">
        <span className="text-[10px] uppercase tracking-wider text-white/30 font-medium">
          Comments ({comments.length})
        </span>
        {comments.length > 0 && (
          <button
            type="button"
            onClick={handleSelectAll}
            className="text-[10px] text-white/40 hover:text-white/60 transition-colors"
          >
            {selectedIds.size === comments.length ? "Deselect all" : "Select all"}
          </button>
        )}
      </div>

      {/* Comments list */}
      <ScrollArea className="flex-1">
        {comments.length === 0 ? (
          <div className="p-4 text-center">
            <p className="text-xs text-white/30">No comments yet</p>
            <p className="text-[10px] text-white/20 mt-1">
              Click on line numbers to add comments
            </p>
          </div>
        ) : (
          <div className="p-1.5 space-y-1">
            {comments.map((comment) => {
              const isSelected = selectedIds.has(comment.id);
              const fileName = comment.file_path.split("/").pop() ?? comment.file_path;

              return (
                <div
                  key={comment.id}
                  className={cn(
                    "group relative rounded-md transition-all duration-100",
                    isSelected ? "bg-white/[0.08]" : "hover:bg-white/[0.04]",
                  )}
                >
                  <div className="flex items-start gap-2 p-2">
                    {/* Checkbox */}
                    <button
                      type="button"
                      onClick={() => handleToggleSelect(comment.id)}
                      className={cn(
                        "w-4 h-4 mt-0.5 rounded border flex items-center justify-center shrink-0",
                        "transition-colors",
                        isSelected
                          ? "bg-blue-500/80 border-blue-500"
                          : "border-white/20 hover:border-white/40",
                      )}
                    >
                      {isSelected && (
                        <svg className="w-2.5 h-2.5 text-white" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                          <path d="M20 6L9 17l-5-5" />
                        </svg>
                      )}
                    </button>

                    {/* Content */}
                    <button
                      type="button"
                      onClick={() => onCommentClick?.(comment)}
                      className="flex-1 text-left min-w-0"
                    >
                      <div className="flex items-center gap-1.5 text-[10px]">
                        <span className="text-white/50 truncate">{fileName}</span>
                        <span className="text-white/20">:</span>
                        <span className="text-blue-400/70 font-mono">
                          {formatLineRange(comment.line_start, comment.line_end)}
                        </span>
                        <span
                          className={cn(
                            "px-1 py-0.5 rounded text-[9px]",
                            comment.side === "new"
                              ? "bg-emerald-500/10 text-emerald-400/50"
                              : "bg-red-500/10 text-red-400/50",
                          )}
                        >
                          {comment.side}
                        </span>
                      </div>
                      <p className="text-xs text-white/70 mt-1 line-clamp-2">
                        {comment.content}
                      </p>
                      {comment.status !== "pending" && (
                        <span
                          className={cn(
                            "inline-block mt-1 px-1.5 py-0.5 rounded text-[9px] font-medium",
                            comment.status === "sent"
                              ? "bg-amber-500/10 text-amber-400/60"
                              : "bg-emerald-500/10 text-emerald-400/60",
                          )}
                        >
                          {comment.status}
                        </span>
                      )}
                    </button>

                    {/* Delete button */}
                    <button
                      type="button"
                      onClick={() => handleDelete(comment.id)}
                      className={cn(
                        "w-5 h-5 rounded flex items-center justify-center shrink-0",
                        "text-white/20 hover:text-red-400/70 hover:bg-red-500/10",
                        "opacity-0 group-hover:opacity-100 transition-all",
                      )}
                    >
                      <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <path d="M18 6L6 18M6 6l12 12" />
                      </svg>
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </ScrollArea>

      {/* Footer - Send to AI */}
      {comments.length > 0 && (
        <div className="p-3 border-t border-white/[0.06]">
          <button
            type="button"
            onClick={handleSendToFix}
            disabled={selectedIds.size === 0 || isSending}
            className={cn(
              "w-full py-2 px-3 rounded-md text-xs font-medium",
              "flex items-center justify-center gap-2",
              "transition-all",
              selectedIds.size > 0 && !isSending
                ? "bg-blue-500/80 text-white hover:bg-blue-500"
                : "bg-white/5 text-white/30 cursor-not-allowed",
            )}
          >
            {isSending ? (
              <>
                <svg className="w-3.5 h-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="2" className="opacity-25" />
                  <path d="M4 12a8 8 0 018-8" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
                </svg>
                Sending...
              </>
            ) : (
              <>
                <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z" />
                </svg>
                Send {selectedIds.size > 0 ? `${selectedIds.size} comment${selectedIds.size > 1 ? "s" : ""}` : "selected"} to AI Fix
              </>
            )}
          </button>
        </div>
      )}
    </aside>
  );
}
