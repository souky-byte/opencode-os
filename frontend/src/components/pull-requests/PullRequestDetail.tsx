import {
  useGetPullRequest,
  useGetPullRequestDiff,
  useGetPullRequestComments,
} from "@/api/generated/pull-requests/pull-requests";
import { Loader } from "@/components/ui/loader";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { usePullRequestStore } from "@/stores/usePullRequestStore";
import { PullRequestHeader } from "./PullRequestHeader";
import { PrConversationTab } from "./PrConversationTab";
import { PrFilesTab } from "./PrFilesTab";
import { PrDiffTab } from "./PrDiffTab";
import { PrCommentsSidebar } from "./PrCommentsSidebar";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { MessageSquare, Files, GitCompare } from "lucide-react";

interface PullRequestDetailProps {
  prNumber: number;
}

function PullRequestDetail({ prNumber }: PullRequestDetailProps) {
  const { activeTab, setActiveTab } = usePullRequestStore();

  const {
    data: prData,
    isLoading: isPrLoading,
    error: prError,
  } = useGetPullRequest(prNumber);

  const { data: diffData, isLoading: isDiffLoading } =
    useGetPullRequestDiff(prNumber);

  const { data: commentsData, isLoading: isCommentsLoading } =
    useGetPullRequestComments(prNumber);

  if (isPrLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader message="Loading pull request..." />
      </div>
    );
  }

  if (prError || !prData?.data) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-destructive">Failed to load pull request</p>
          <p className="text-sm text-muted-foreground">
            Pull request #{prNumber} not found
          </p>
        </div>
      </div>
    );
  }

  const pr = prData.data;
  const reviewComments = commentsData?.data?.review_comments ?? [];
  const issueComments = commentsData?.data?.issue_comments ?? [];

  return (
    <ResizablePanelGroup direction="horizontal" className="h-full">
      <ResizablePanel defaultSize={70} minSize={50}>
        <div className="flex flex-col h-full">
          <PullRequestHeader pr={pr} />

          <Tabs
            value={activeTab}
            onValueChange={(v) =>
              setActiveTab(v as "conversation" | "files" | "diff")
            }
            className="flex-1 flex flex-col min-h-0"
          >
            <TabsList className="mx-4 mt-2 w-fit">
              <TabsTrigger value="conversation" className="text-xs gap-1.5">
                <MessageSquare className="w-3.5 h-3.5" />
                Conversation
                {issueComments.length > 0 && (
                  <span className="ml-1 px-1.5 py-0.5 text-[10px] bg-muted rounded-full">
                    {issueComments.length}
                  </span>
                )}
              </TabsTrigger>
              <TabsTrigger value="files" className="text-xs gap-1.5">
                <Files className="w-3.5 h-3.5" />
                Files
                <span className="ml-1 px-1.5 py-0.5 text-[10px] bg-muted rounded-full">
                  {pr.changed_files}
                </span>
              </TabsTrigger>
              <TabsTrigger value="diff" className="text-xs gap-1.5">
                <GitCompare className="w-3.5 h-3.5" />
                Diff
              </TabsTrigger>
            </TabsList>

            <TabsContent
              value="conversation"
              className="flex-1 overflow-hidden m-0"
            >
              <PrConversationTab
                pr={pr}
                issueComments={issueComments}
                isLoading={isCommentsLoading}
              />
            </TabsContent>

            <TabsContent value="files" className="flex-1 overflow-hidden m-0">
              <PrFilesTab prNumber={prNumber} />
            </TabsContent>

            <TabsContent value="diff" className="flex-1 overflow-hidden m-0">
              <PrDiffTab
                diff={diffData?.data?.diff}
                isLoading={isDiffLoading}
              />
            </TabsContent>
          </Tabs>
        </div>
      </ResizablePanel>

      {reviewComments.length > 0 && (
        <>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize={30} minSize={20} maxSize={40}>
            <PrCommentsSidebar
              comments={reviewComments}
              prNumber={prNumber}
              isLoading={isCommentsLoading}
            />
          </ResizablePanel>
        </>
      )}
    </ResizablePanelGroup>
  );
}

export { PullRequestDetail };
