import { useState } from "react";
import {
  useGetWorkspaceDiff,
  useGetViewedFiles,
  useSetFileViewed,
} from "@/api/generated/workspaces/workspaces";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Loader } from "@/components/ui/loader";
import { DiffContent } from "./DiffContent";
import {
  useDiffParser,
  getFileDisplayName,
  getFileExtension,
} from "./useDiffParser";
import { cn } from "@/lib/utils";

interface DiffOverlayProps {
  taskId: string;
  isOpen: boolean;
  onClose: () => void;
}

export function DiffOverlay({ taskId, isOpen, onClose }: DiffOverlayProps) {
  const [selectedFileIndex, setSelectedFileIndex] = useState(0);

  const { data: diffData, isLoading: isDiffLoading } = useGetWorkspaceDiff(
    taskId,
    {
      query: { staleTime: 10000, enabled: isOpen },
    },
  );

  const { data: viewedData } = useGetViewedFiles(taskId, {
    query: { staleTime: 5000, enabled: isOpen },
  });

  const { mutate: setViewed } = useSetFileViewed();

  const parsedDiff = useDiffParser(diffData?.data?.diff);
  const viewedFiles = viewedData?.data?.viewed_files ?? [];

  const selectedFile = parsedDiff?.files[selectedFileIndex] ?? null;

  const handleToggleViewed = (filePath: string, viewed: boolean) => {
    setViewed({ taskId, data: { file_path: filePath, viewed } });
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      onClose();
    } else if (e.key === "ArrowDown" || e.key === "j") {
      e.preventDefault();
      if (parsedDiff && selectedFileIndex < parsedDiff.files.length - 1) {
        setSelectedFileIndex(selectedFileIndex + 1);
      }
    } else if (e.key === "ArrowUp" || e.key === "k") {
      e.preventDefault();
      if (selectedFileIndex > 0) {
        setSelectedFileIndex(selectedFileIndex - 1);
      }
    }
  };

  const viewedCount =
    parsedDiff?.files.filter((f) => viewedFiles.includes(getFileDisplayName(f)))
      .length ?? 0;

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 bg-[#0d0d12] flex flex-col"
      onKeyDown={handleKeyDown}
      tabIndex={0}
    >
      {/* Header */}
      <header className="flex items-center justify-between px-5 py-3 border-b border-white/[0.06] bg-[#111117]">
        <div className="flex items-center gap-4">
          <button
            type="button"
            onClick={onClose}
            className="flex items-center gap-2 text-sm text-white/50 hover:text-white/80 transition-colors"
          >
            <svg
              className="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
            >
              <path d="M19 12H5M12 19l-7-7 7-7" />
            </svg>
            <span className="font-medium">Back</span>
          </button>
          <div className="w-px h-4 bg-white/10" />
          <h1 className="text-sm font-medium text-white/70">Review Changes</h1>
        </div>

        <div className="flex items-center gap-6 text-xs">
          <div className="flex items-center gap-3 text-white/40">
            <span>
              {viewedCount}/{parsedDiff?.fileCount ?? 0} reviewed
            </span>
            <div className="w-24 h-1 bg-white/10 rounded-full overflow-hidden">
              <div
                className="h-full bg-emerald-500/60 transition-all duration-300"
                style={{
                  width: `${parsedDiff?.fileCount ? (viewedCount / parsedDiff.fileCount) * 100 : 0}%`,
                }}
              />
            </div>
          </div>
          <div className="flex items-center gap-3 font-mono">
            <span className="text-emerald-400/70">
              +{parsedDiff?.totalAdditions ?? 0}
            </span>
            <span className="text-red-400/70">
              -{parsedDiff?.totalDeletions ?? 0}
            </span>
          </div>
          <div className="text-white/30 text-[10px]">
            <kbd className="px-1.5 py-0.5 bg-white/5 rounded border border-white/10">
              ↑↓
            </kbd>{" "}
            navigate
            <span className="mx-2">·</span>
            <kbd className="px-1.5 py-0.5 bg-white/5 rounded border border-white/10">
              esc
            </kbd>{" "}
            close
          </div>
        </div>
      </header>

      {/* Main content */}
      {isDiffLoading ? (
        <div className="flex-1 flex items-center justify-center">
          <Loader />
        </div>
      ) : !parsedDiff || parsedDiff.files.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-white/40">
            <p className="text-sm">No changes to review</p>
            <p className="text-xs mt-1 text-white/25">
              Make some changes in the workspace first
            </p>
          </div>
        </div>
      ) : (
        <div className="flex-1 flex min-h-0">
          {/* File list sidebar */}
          <aside className="w-72 border-r border-white/[0.06] bg-[#0a0a0f] flex flex-col">
            <div className="px-3 py-2 border-b border-white/[0.04]">
              <span className="text-[10px] uppercase tracking-wider text-white/30 font-medium">
                Changed files
              </span>
            </div>
            <ScrollArea className="flex-1">
              <div className="p-1.5">
                {parsedDiff.files.map((file, index) => {
                  const filePath = getFileDisplayName(file);
                  const fileName = filePath.split("/").pop() ?? filePath;
                  const dirPath = filePath.split("/").slice(0, -1).join("/");
                  const isViewed = viewedFiles.includes(filePath);
                  const isSelected = index === selectedFileIndex;
                  const ext = getFileExtension(file);

                  return (
                    <button
                      key={filePath}
                      type="button"
                      onClick={() => setSelectedFileIndex(index)}
                      className={cn(
                        "w-full text-left px-2.5 py-2 rounded-md transition-all duration-100",
                        "group relative",
                        isSelected
                          ? "bg-white/[0.08]"
                          : "hover:bg-white/[0.04]",
                        isViewed && "opacity-50",
                      )}
                    >
                      <div className="flex items-start gap-2.5">
                        <FileIcon
                          ext={ext}
                          isNew={file.isNew}
                          isDeleted={file.isDeleted}
                        />
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span
                              className={cn(
                                "text-xs font-medium truncate",
                                isSelected ? "text-white/90" : "text-white/70",
                              )}
                            >
                              {fileName}
                            </span>
                            {isViewed && (
                              <svg
                                className="w-3 h-3 text-emerald-500/60 shrink-0"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                strokeWidth="2"
                              >
                                <path d="M20 6L9 17l-5-5" />
                              </svg>
                            )}
                          </div>
                          {dirPath && (
                            <span className="text-[10px] text-white/30 truncate block">
                              {dirPath}
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-1.5 text-[10px] font-mono shrink-0">
                          {file.addedLines > 0 && (
                            <span className="text-emerald-400/60">
                              +{file.addedLines}
                            </span>
                          )}
                          {file.deletedLines > 0 && (
                            <span className="text-red-400/60">
                              -{file.deletedLines}
                            </span>
                          )}
                        </div>
                      </div>
                    </button>
                  );
                })}
              </div>
            </ScrollArea>
          </aside>

          {/* Diff content */}
          <main className="flex-1 flex flex-col min-w-0 bg-[#0d0d12]">
            {selectedFile && (
              <>
                {/* File header */}
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/[0.06] bg-[#111117]">
                  <div className="flex items-center gap-3 min-w-0">
                    <span className="font-mono text-xs text-white/60 truncate">
                      {getFileDisplayName(selectedFile)}
                    </span>
                    <div className="flex items-center gap-2 text-[10px] font-mono shrink-0">
                      {selectedFile.addedLines > 0 && (
                        <span className="text-emerald-400/70">
                          +{selectedFile.addedLines}
                        </span>
                      )}
                      {selectedFile.deletedLines > 0 && (
                        <span className="text-red-400/70">
                          -{selectedFile.deletedLines}
                        </span>
                      )}
                    </div>
                  </div>
                  <label className="flex items-center gap-2 text-xs text-white/40 cursor-pointer hover:text-white/60 transition-colors">
                    <input
                      type="checkbox"
                      checked={viewedFiles.includes(
                        getFileDisplayName(selectedFile),
                      )}
                      onChange={() =>
                        handleToggleViewed(
                          getFileDisplayName(selectedFile),
                          !viewedFiles.includes(
                            getFileDisplayName(selectedFile),
                          ),
                        )
                      }
                      className="w-3.5 h-3.5 rounded border-white/20 bg-white/5 checked:bg-emerald-500 checked:border-emerald-500"
                    />
                    Mark as reviewed
                  </label>
                </div>
                {/* Diff view */}
                <ScrollArea className="flex-1">
                  <DiffContent file={selectedFile} />
                </ScrollArea>
              </>
            )}
          </main>
        </div>
      )}
    </div>
  );
}

// File type icon component
function FileIcon({
  ext,
  isNew,
  isDeleted,
}: {
  ext: string;
  isNew?: boolean;
  isDeleted?: boolean;
}) {
  let color = "text-white/30";

  if (isNew) color = "text-emerald-400/60";
  else if (isDeleted) color = "text-red-400/60";
  else if (["ts", "tsx"].includes(ext)) color = "text-blue-400/60";
  else if (["js", "jsx"].includes(ext)) color = "text-yellow-400/60";
  else if (["css", "scss"].includes(ext)) color = "text-pink-400/60";
  else if (["json"].includes(ext)) color = "text-orange-400/60";
  else if (["md", "mdx"].includes(ext)) color = "text-white/40";

  return (
    <div className={cn("w-4 h-4 shrink-0 mt-0.5", color)}>
      <svg
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
      >
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <polyline points="14 2 14 8 20 8" />
      </svg>
    </div>
  );
}
