import { useMemo } from "react";
import { useListPullRequests } from "@/api/generated/pull-requests/pull-requests";
import { Loader } from "@/components/ui/loader";
import { ScrollArea } from "@/components/ui/scroll-area";
import { usePullRequestStore } from "@/stores/usePullRequestStore";
import { PullRequestCard } from "./PullRequestCard";
import { PullRequestFilters } from "./PullRequestFilters";
import { GitPullRequest } from "lucide-react";

function PullRequestsList() {
  const { filters, sort, selectedPrNumber, selectPr } = usePullRequestStore();

  const { data, isLoading, error } = useListPullRequests(
    filters.state === "all"
      ? undefined
      : {
          state: filters.state,
        },
    {
      query: {
        refetchInterval: 30000, // Poll every 30 seconds
      },
    },
  );

  const sortedPrs = useMemo(() => {
    const prs = data?.data?.pull_requests ?? [];

    return [...prs].sort((a, b) => {
      const dateA = new Date(
        sort.field === "updated" ? a.updated_at : a.created_at,
      );
      const dateB = new Date(
        sort.field === "updated" ? b.updated_at : b.created_at,
      );

      return sort.order === "desc"
        ? dateB.getTime() - dateA.getTime()
        : dateA.getTime() - dateB.getTime();
    });
  }, [data, sort]);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader message="Loading pull requests..." />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-destructive">Failed to load pull requests</p>
          <p className="text-sm text-muted-foreground">
            Please check your GitHub connection
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <PullRequestFilters />

      {sortedPrs.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-muted-foreground">
            <GitPullRequest className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No pull requests found</p>
          </div>
        </div>
      ) : (
        <ScrollArea className="flex-1">
          <div className="flex flex-col gap-2 p-2">
            {sortedPrs.map((pr) => (
              <PullRequestCard
                key={pr.number}
                pr={pr}
                isSelected={pr.number === selectedPrNumber}
                onClick={() => selectPr(pr.number)}
              />
            ))}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}

export { PullRequestsList };
