import { usePullRequestStore } from "@/stores/usePullRequestStore";
import { PullRequestsList } from "./PullRequestsList";
import { PullRequestDetail } from "./PullRequestDetail";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";

function PullRequestsView() {
  const { selectedPrNumber } = usePullRequestStore();

  return (
    <ResizablePanelGroup direction="horizontal" className="h-full">
      <ResizablePanel
        defaultSize={selectedPrNumber ? 35 : 100}
        minSize={25}
        maxSize={50}
      >
        <PullRequestsList />
      </ResizablePanel>

      {selectedPrNumber && (
        <>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize={65} minSize={50}>
            <PullRequestDetail prNumber={selectedPrNumber} />
          </ResizablePanel>
        </>
      )}
    </ResizablePanelGroup>
  );
}

export { PullRequestsView };
