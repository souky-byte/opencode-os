import type { PullRequestDetail, CiStatus } from "@/api/generated/model";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  GitPullRequest,
  GitMerge,
  ExternalLink,
  X,
  Check,
  Clock,
  AlertCircle,
} from "lucide-react";
import { formatDistanceToNow } from "@/lib/date";
import { usePullRequestStore } from "@/stores/usePullRequestStore";

interface PullRequestHeaderProps {
  pr: PullRequestDetail;
}

function getCiStatusBadge(ciStatus?: CiStatus | null) {
  if (!ciStatus) return null;
  switch (ciStatus.state) {
    case "success":
      return (
        <Badge className="bg-emerald-500/10 text-emerald-500 border-emerald-500/20">
          <Check className="w-3 h-3 mr-1" />
          Checks passed
        </Badge>
      );
    case "failure":
    case "error":
      return (
        <Badge className="bg-red-500/10 text-red-500 border-red-500/20">
          <X className="w-3 h-3 mr-1" />
          Checks failed
        </Badge>
      );
    case "pending":
      return (
        <Badge className="bg-yellow-500/10 text-yellow-500 border-yellow-500/20">
          <Clock className="w-3 h-3 mr-1 animate-pulse" />
          Checks pending
        </Badge>
      );
    default:
      return (
        <Badge variant="outline">
          <AlertCircle className="w-3 h-3 mr-1" />
          Unknown status
        </Badge>
      );
  }
}

function PullRequestHeader({ pr }: PullRequestHeaderProps) {
  const { selectPr } = usePullRequestStore();
  const isMerged = !!pr.merged_at;
  const isClosed = pr.state === "closed" && !isMerged;

  return (
    <div className="border-b bg-card/50 p-4">
      {/* Top row: close button and external link */}
      <div className="flex items-center justify-between mb-3">
        <Button
          variant="ghost"
          size="sm"
          className="h-7 px-2"
          onClick={() => selectPr(null)}
        >
          <X className="w-4 h-4" />
        </Button>
        <a
          href={pr.html_url}
          target="_blank"
          rel="noopener noreferrer"
          className="text-muted-foreground hover:text-foreground transition-colors"
        >
          <ExternalLink className="w-4 h-4" />
        </a>
      </div>

      {/* Title row */}
      <div className="flex items-start gap-3">
        <div className="shrink-0 mt-1">
          {isMerged ? (
            <GitMerge className="w-5 h-5 text-purple-500" />
          ) : isClosed ? (
            <GitPullRequest className="w-5 h-5 text-red-500" />
          ) : (
            <GitPullRequest className="w-5 h-5 text-emerald-500" />
          )}
        </div>
        <div className="flex-1 min-w-0">
          <h2 className="text-lg font-semibold leading-tight">{pr.title}</h2>
          <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
            <span className="font-medium">#{pr.number}</span>
            <span>|</span>
            <span>
              {pr.head_branch} → {pr.base_branch}
            </span>
          </div>
        </div>
      </div>

      {/* Meta row */}
      <div className="flex flex-wrap items-center gap-3 mt-4">
        {/* Author */}
        <div className="flex items-center gap-2">
          <img
            src={pr.user.avatar_url}
            alt={pr.user.login}
            className="w-5 h-5 rounded-full"
          />
          <span className="text-sm">{pr.user.login}</span>
          <span className="text-sm text-muted-foreground">
            opened {formatDistanceToNow(new Date(pr.created_at))}
          </span>
        </div>

        {/* Divider */}
        <span className="text-muted-foreground/30">|</span>

        {/* Stats */}
        <div className="flex items-center gap-2 text-sm">
          <span className="text-emerald-500 font-medium">+{pr.additions}</span>
          <span className="text-red-500 font-medium">-{pr.deletions}</span>
          <span className="text-muted-foreground">
            in {pr.changed_files} files
          </span>
        </div>

        {/* CI Status */}
        {getCiStatusBadge(pr.ci_status)}

        {/* Draft badge */}
        {pr.draft && (
          <Badge variant="outline" className="text-xs">
            Draft
          </Badge>
        )}

        {/* Merge status badges */}
        {isMerged && (
          <Badge className="bg-purple-500/10 text-purple-500 border-purple-500/20">
            <GitMerge className="w-3 h-3 mr-1" />
            Merged
          </Badge>
        )}
        {isClosed && (
          <Badge className="bg-red-500/10 text-red-500 border-red-500/20">
            Closed
          </Badge>
        )}
      </div>

      {/* Labels */}
      {pr.labels.length > 0 && (
        <div className="flex flex-wrap gap-1.5 mt-3">
          {pr.labels.map((label) => (
            <Badge
              key={label.name}
              variant="secondary"
              className="text-xs"
              style={{
                backgroundColor: `#${label.color}20`,
                borderColor: `#${label.color}40`,
                color: `#${label.color}`,
              }}
            >
              {label.name}
            </Badge>
          ))}
        </div>
      )}

      {/* Reviewers */}
      {pr.requested_reviewers.length > 0 && (
        <div className="flex items-center gap-2 mt-3">
          <span className="text-xs text-muted-foreground">Reviewers:</span>
          <div className="flex items-center -space-x-1">
            {pr.requested_reviewers.map((reviewer) => (
              <img
                key={reviewer.login}
                src={reviewer.avatar_url}
                alt={reviewer.login}
                title={reviewer.login}
                className="w-5 h-5 rounded-full border-2 border-background"
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

export { PullRequestHeader };
