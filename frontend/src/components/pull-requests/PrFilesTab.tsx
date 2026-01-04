import { useGetPullRequestFiles } from "@/api/generated/pull-requests/pull-requests";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Loader } from "@/components/ui/loader";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import { File, FilePlus, FileMinus, FileEdit } from "lucide-react";

interface PrFilesTabProps {
  prNumber: number;
}

function getFileIcon(status: string) {
  switch (status) {
    case "added":
      return <FilePlus className="w-4 h-4 text-emerald-500" />;
    case "removed":
      return <FileMinus className="w-4 h-4 text-red-500" />;
    case "modified":
      return <FileEdit className="w-4 h-4 text-yellow-500" />;
    case "renamed":
      return <FileEdit className="w-4 h-4 text-blue-500" />;
    default:
      return <File className="w-4 h-4 text-muted-foreground" />;
  }
}

function getStatusBadge(status: string) {
  switch (status) {
    case "added":
      return (
        <Badge className="text-[9px] px-1 py-0 h-4 bg-emerald-500/10 text-emerald-500 border-emerald-500/20">
          Added
        </Badge>
      );
    case "removed":
      return (
        <Badge className="text-[9px] px-1 py-0 h-4 bg-red-500/10 text-red-500 border-red-500/20">
          Removed
        </Badge>
      );
    case "modified":
      return (
        <Badge className="text-[9px] px-1 py-0 h-4 bg-yellow-500/10 text-yellow-500 border-yellow-500/20">
          Modified
        </Badge>
      );
    case "renamed":
      return (
        <Badge className="text-[9px] px-1 py-0 h-4 bg-blue-500/10 text-blue-500 border-blue-500/20">
          Renamed
        </Badge>
      );
    default:
      return null;
  }
}

function PrFilesTab({ prNumber }: PrFilesTabProps) {
  const { data, isLoading, error } = useGetPullRequestFiles(prNumber);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader message="Loading files..." />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-destructive">Failed to load files</p>
      </div>
    );
  }

  const files = data?.data ?? [];

  if (files.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-muted-foreground">No files changed</p>
      </div>
    );
  }

  // Group files by directory
  const filesByDir = files.reduce(
    (acc, file) => {
      const parts = file.filename.split("/");
      const dir = parts.length > 1 ? parts.slice(0, -1).join("/") : "/";
      if (!acc[dir]) {
        acc[dir] = [];
      }
      acc[dir].push(file);
      return acc;
    },
    {} as Record<string, typeof files>,
  );

  return (
    <ScrollArea className="h-full">
      <div className="p-4">
        {/* Summary */}
        <div className="flex items-center gap-4 mb-4 text-sm">
          <span className="text-muted-foreground">
            {files.length} files changed
          </span>
          <span className="text-emerald-500">
            +{files.reduce((sum, f) => sum + f.additions, 0)}
          </span>
          <span className="text-red-500">
            -{files.reduce((sum, f) => sum + f.deletions, 0)}
          </span>
        </div>

        {/* File list */}
        <div className="space-y-4">
          {Object.entries(filesByDir).map(([dir, dirFiles]) => (
            <div key={dir}>
              {dir !== "/" && (
                <div className="text-xs text-muted-foreground mb-2 font-mono">
                  {dir}/
                </div>
              )}
              <div className="space-y-1">
                {dirFiles.map((file) => {
                  const fileName = file.filename.split("/").pop();
                  return (
                    <div
                      key={file.filename}
                      className={cn(
                        "flex items-center gap-3 p-2 rounded-lg",
                        "hover:bg-accent/50 cursor-pointer transition-colors",
                      )}
                    >
                      {getFileIcon(file.status)}
                      <span className="flex-1 text-sm font-mono truncate">
                        {fileName}
                      </span>
                      {getStatusBadge(file.status)}
                      <div className="flex items-center gap-2 text-xs font-mono">
                        <span className="text-emerald-500">
                          +{file.additions}
                        </span>
                        <span className="text-red-500">-{file.deletions}</span>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      </div>
    </ScrollArea>
  );
}

export { PrFilesTab };
