import { ScrollArea } from "@/components/ui/scroll-area";
import { Loader } from "@/components/ui/loader";
import {
  useDiffParser,
  getFileDisplayName,
} from "@/components/diff/useDiffParser";
import { DiffFileSection } from "@/components/diff/DiffFileSection";

interface PrDiffTabProps {
  diff?: string;
  isLoading: boolean;
}

function PrDiffTab({ diff, isLoading }: PrDiffTabProps) {
  const parsedDiff = useDiffParser(diff);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader message="Loading diff..." />
      </div>
    );
  }

  if (!parsedDiff || parsedDiff.files.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center text-muted-foreground">
          <p>No diff available</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Summary header */}
      <div className="flex items-center gap-4 px-4 py-2 border-b text-sm">
        <span className="text-muted-foreground">
          {parsedDiff.fileCount} files
        </span>
        <span className="text-emerald-500">+{parsedDiff.totalAdditions}</span>
        <span className="text-red-500">-{parsedDiff.totalDeletions}</span>
      </div>

      <ScrollArea className="flex-1">
        <div className="divide-y divide-border">
          {parsedDiff.files.map((file) => {
            const filePath = getFileDisplayName(file);
            return (
              <DiffFileSection
                key={filePath}
                file={file}
                isViewed={false}
                onToggleViewed={() => {}}
              />
            );
          })}
        </div>
      </ScrollArea>
    </div>
  );
}

export { PrDiffTab };
