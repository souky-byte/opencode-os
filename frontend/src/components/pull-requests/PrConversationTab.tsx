import type { PullRequestDetail, PrIssueComment } from "@/api/generated/model";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Loader } from "@/components/ui/loader";
import { Markdown } from "@/components/ui/markdown";
import { formatDistanceToNow } from "@/lib/date";

interface PrConversationTabProps {
  pr: PullRequestDetail;
  issueComments: PrIssueComment[];
  isLoading: boolean;
}

function PrConversationTab({
  pr,
  issueComments,
  isLoading,
}: PrConversationTabProps) {
  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader message="Loading comments..." />
      </div>
    );
  }

  return (
    <ScrollArea className="h-full">
      <div className="p-4 space-y-4">
        {/* PR Description */}
        {pr.body && (
          <div className="rounded-lg border bg-card p-4">
            <div className="flex items-center gap-2 mb-3">
              <img
                src={pr.user.avatar_url}
                alt={pr.user.login}
                className="w-6 h-6 rounded-full"
              />
              <span className="font-medium text-sm">{pr.user.login}</span>
              <span className="text-xs text-muted-foreground">
                {formatDistanceToNow(new Date(pr.created_at))}
              </span>
            </div>
            <div className="text-sm">
              <Markdown text={pr.body} />
            </div>
          </div>
        )}

        {/* Issue Comments */}
        {issueComments.map((comment) => (
          <div key={comment.id} className="rounded-lg border bg-card p-4">
            <div className="flex items-center gap-2 mb-3">
              <img
                src={comment.user.avatar_url}
                alt={comment.user.login}
                className="w-6 h-6 rounded-full"
              />
              <span className="font-medium text-sm">{comment.user.login}</span>
              <span className="text-xs text-muted-foreground">
                {formatDistanceToNow(new Date(comment.created_at))}
              </span>
              {comment.reactions && (
                <div className="flex items-center gap-1 ml-auto text-xs text-muted-foreground">
                  {comment.reactions.total_count > 0 && (
                    <span>+{comment.reactions.total_count}</span>
                  )}
                </div>
              )}
            </div>
            <div className="text-sm">
              <Markdown text={comment.body} />
            </div>
          </div>
        ))}

        {!pr.body && issueComments.length === 0 && (
          <div className="text-center text-muted-foreground py-8">
            <p>No conversation yet</p>
          </div>
        )}
      </div>
    </ScrollArea>
  );
}

export { PrConversationTab };
