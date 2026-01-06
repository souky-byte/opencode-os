import type { PullRequestDetail, CiStatus } from "@/api/generated/model";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import {
  GitPullRequest,
  GitMerge,
  MessageSquare,
  Check,
  X,
  Clock,
  AlertCircle,
} from "lucide-react";
import { formatDistanceToNow } from "@/lib/date";

interface PullRequestCardProps {
  pr: PullRequestDetail;
  isSelected?: boolean;
  onClick?: () => void;
}

function getCiStatusIcon(ciStatus?: CiStatus | null) {
  if (!ciStatus) return null;
  switch (ciStatus.state) {
    case "success":
      return <Check className="w-3 h-3 text-emerald-500" />;
    case "failure":
    case "error":
      return <X className="w-3 h-3 text-red-500" />;
    case "pending":
      return <Clock className="w-3 h-3 text-yellow-500 animate-pulse" />;
    default:
      return <AlertCircle className="w-3 h-3 text-muted-foreground" />;
  }
}

function PullRequestCard({ pr, isSelected, onClick }: PullRequestCardProps) {
  const isMerged = !!pr.merged_at;
  const isClosed = pr.state === "closed" && !isMerged;
  const isDraft = pr.draft;

  return (
    <div
      className={cn(
        "group relative rounded-lg border bg-card p-3 cursor-pointer",
        "transition-all duration-150 ease-out",
        "hover:bg-accent/50 hover:border-border/80",
        isSelected && "border-primary bg-primary/5 ring-1 ring-primary",
      )}
      onClick={onClick}
      onKeyDown={(e) => e.key === "Enter" && onClick?.()}
      tabIndex={0}
      role="button"
    >
      <div className="flex items-start gap-2">
        {/* PR Icon */}
        <div className="shrink-0 mt-0.5">
          {isMerged ? (
            <GitMerge className="w-4 h-4 text-purple-500" />
          ) : isClosed ? (
            <GitPullRequest className="w-4 h-4 text-red-500" />
          ) : (
            <GitPullRequest className="w-4 h-4 text-emerald-500" />
          )}
        </div>

        <div className="flex-1 min-w-0">
          {/* Title row */}
          <div className="flex items-start justify-between gap-2">
            <div className="flex items-center gap-1.5 min-w-0 flex-1">
              {getCiStatusIcon(pr.ci_status)}
              <h4 className="text-xs font-medium leading-tight line-clamp-2 text-foreground">
                {pr.title}
              </h4>
            </div>
            {isDraft && (
              <Badge
                variant="outline"
                className="text-[9px] px-1 py-0 h-4 shrink-0"
              >
                Draft
              </Badge>
            )}
          </div>

          {/* Meta row */}
          <div className="flex items-center gap-2 mt-1.5 text-[10px] text-muted-foreground">
            <span className="font-medium">#{pr.number}</span>
            <span className="text-muted-foreground/50">|</span>
            <span className="truncate">
              {pr.head_branch} → {pr.base_branch}
            </span>
            <span className="text-muted-foreground/50">|</span>
            <span>{formatDistanceToNow(new Date(pr.updated_at))}</span>
          </div>

          {/* Stats row */}
          <div className="flex items-center gap-3 mt-2">
            {/* Author */}
            <div className="flex items-center gap-1">
              <img
                src={pr.user.avatar_url}
                alt={pr.user.login}
                className="w-4 h-4 rounded-full"
              />
              <span className="text-[10px] text-muted-foreground">
                {pr.user.login}
              </span>
            </div>

            {/* Diff stats */}
            <div className="flex items-center gap-1 text-[10px]">
              <span className="text-emerald-500">+{pr.additions}</span>
              <span className="text-red-500">-{pr.deletions}</span>
            </div>

            {/* Comments */}
            {(pr.comments_count > 0 || pr.review_comments_count > 0) && (
              <div className="flex items-center gap-1 text-[10px] text-muted-foreground">
                <MessageSquare className="w-3 h-3" />
                <span>{pr.comments_count + pr.review_comments_count}</span>
              </div>
            )}

            {/* Reviewers */}
            {pr.requested_reviewers.length > 0 && (
              <div className="flex items-center -space-x-1">
                {pr.requested_reviewers.slice(0, 3).map((reviewer) => (
                  <img
                    key={reviewer.login}
                    src={reviewer.avatar_url}
                    alt={reviewer.login}
                    title={reviewer.login}
                    className="w-4 h-4 rounded-full border border-background"
                  />
                ))}
                {pr.requested_reviewers.length > 3 && (
                  <span className="text-[9px] text-muted-foreground ml-1">
                    +{pr.requested_reviewers.length - 3}
                  </span>
                )}
              </div>
            )}
          </div>

          {/* Labels */}
          {pr.labels.length > 0 && (
            <div className="flex flex-wrap gap-1 mt-2">
              {pr.labels.slice(0, 4).map((label) => (
                <Badge
                  key={label.name}
                  variant="secondary"
                  className="text-[9px] px-1.5 py-0 h-4"
                  style={{
                    backgroundColor: `#${label.color}20`,
                    borderColor: `#${label.color}40`,
                    color: `#${label.color}`,
                  }}
                >
                  {label.name}
                </Badge>
              ))}
              {pr.labels.length > 4 && (
                <span className="text-[9px] text-muted-foreground">
                  +{pr.labels.length - 4}
                </span>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export { PullRequestCard };
