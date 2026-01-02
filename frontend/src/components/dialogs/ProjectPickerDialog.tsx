import NiceModal, { useModal } from "@ebay/nice-modal-react";
import { useQueryClient } from "@tanstack/react-query";
import {
  ChevronRight,
  Folder,
  FolderGit,
  FolderOpen,
  Home,
  X,
} from "lucide-react";
import { useState } from "react";
import { useBrowseDirectory } from "@/api/generated/filesystem/filesystem";
import {
  getGetRecentProjectsQueryKey,
  useClearRecentProjects,
  useGetRecentProjects,
  useOpenProject,
  useRemoveRecentProject,
} from "@/api/generated/projects/projects";
import { getListTasksQueryKey } from "@/api/generated/tasks/tasks";
import { getListSessionsQueryKey } from "@/api/generated/sessions/sessions";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Loader } from "@/components/ui/loader";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import { defineModal } from "@/lib/modals";
import { useProjectStore } from "@/stores/useProjectStore";

interface ProjectPickerDialogProps {
  allowClose?: boolean;
}

const ProjectPickerDialogComponent = NiceModal.create<ProjectPickerDialogProps>(
  ({ allowClose = true }) => {
    const modal = useModal();
    const queryClient = useQueryClient();
    const [currentPath, setCurrentPath] = useState<string | undefined>(
      undefined,
    );
    const [validationError, setValidationError] = useState<string | null>(null);
    const [mode, setMode] = useState<"recent" | "browse">("recent");

    const { setCurrentProject, closeDialog } = useProjectStore();

    const { data: recentProjectsResponse, isLoading: isLoadingRecent } =
      useGetRecentProjects();
    const recentProjects = recentProjectsResponse?.data.projects ?? [];

    const { data: browseResponse, isLoading: isBrowsing } = useBrowseDirectory(
      { path: currentPath },
      { query: { enabled: mode === "browse" } },
    );
    const browseData = browseResponse?.data;

    const openProject = useOpenProject({
      mutation: {
        onSuccess: (response) => {
          if (response.data.success && response.data.project) {
            // Invalidate all task and session queries when switching projects
            // This ensures the UI shows tasks for the new project
            void queryClient.invalidateQueries({
              queryKey: getListTasksQueryKey(),
            });
            void queryClient.invalidateQueries({
              queryKey: getListSessionsQueryKey(),
            });

            setCurrentProject(response.data.project);
            closeDialog();
            modal.resolve(response.data.project);
            void modal.hide();
          } else {
            setValidationError(
              response.data.error?.message ?? "Failed to open project",
            );
          }
        },
        onError: () => {
          setValidationError("Failed to open project");
        },
      },
    });

    const removeRecent = useRemoveRecentProject({
      mutation: {
        onSuccess: () => {
          void queryClient.invalidateQueries({
            queryKey: getGetRecentProjectsQueryKey(),
          });
        },
      },
    });

    const clearRecent = useClearRecentProjects({
      mutation: {
        onSuccess: () => {
          void queryClient.invalidateQueries({
            queryKey: getGetRecentProjectsQueryKey(),
          });
        },
      },
    });

    const handleSelectRecent = (path: string) => {
      setValidationError(null);
      openProject.mutate({ data: { path } });
    };

    const handleRemoveRecent = (e: React.MouseEvent, path: string) => {
      e.stopPropagation();
      removeRecent.mutate({ data: { path } });
    };

    const handleClearAll = () => {
      clearRecent.mutate();
    };

    const handleBrowseFolder = (path: string) => {
      setCurrentPath(path);
      setValidationError(null);
    };

    const handleSelectFolder = (path: string) => {
      setValidationError(null);
      openProject.mutate({ data: { path } });
    };

    const handleOpenChange = (open: boolean) => {
      if (!open && allowClose) {
        closeDialog();
        void modal.hide();
      }
    };

    const isPending = openProject.isPending;

    const pathSegments =
      browseData?.current_path.split("/").filter(Boolean) ?? [];

    return (
      <Dialog open={modal.visible} onOpenChange={handleOpenChange}>
        <DialogContent className="max-w-lg" hideCloseButton={!allowClose}>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <FolderOpen className="h-5 w-5" />
              Open Project
            </DialogTitle>
            <DialogDescription>
              Select a Git or Jujutsu repository to open.
            </DialogDescription>
          </DialogHeader>

          <div className="flex gap-2 border-b pb-2">
            <Button
              variant={mode === "recent" ? "secondary" : "ghost"}
              size="sm"
              onClick={() => setMode("recent")}
            >
              Recent
            </Button>
            <Button
              variant={mode === "browse" ? "secondary" : "ghost"}
              size="sm"
              onClick={() => {
                setMode("browse");
                setCurrentPath(undefined);
              }}
            >
              Browse
            </Button>
          </div>

          {validationError && (
            <p className="text-sm text-destructive bg-destructive/10 p-2 rounded">
              {validationError}
            </p>
          )}

          {mode === "recent" ? (
            <div className="space-y-2">
              {isLoadingRecent ? (
                <div className="flex items-center justify-center py-8">
                  <Loader />
                </div>
              ) : recentProjects.length > 0 ? (
                <>
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium text-muted-foreground">
                      Recent Projects
                    </span>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={handleClearAll}
                      disabled={clearRecent.isPending}
                      className="h-auto py-1 px-2 text-xs text-muted-foreground hover:text-destructive"
                    >
                      Clear all
                    </Button>
                  </div>
                  <ScrollArea className="h-[300px]">
                    <div className="space-y-1 pr-4">
                      {recentProjects.map((project) => (
                        <div
                          key={project.path}
                          className="group flex items-center gap-2 p-2 rounded-md border hover:bg-accent transition-colors"
                        >
                          <FolderGit className="h-4 w-4 text-muted-foreground shrink-0" />
                          <button
                            type="button"
                            onClick={() => handleSelectRecent(project.path)}
                            disabled={isPending}
                            className="flex-1 min-w-0 text-left disabled:opacity-50"
                          >
                            <div className="font-medium truncate text-sm">
                              {project.name}
                            </div>
                            <div className="text-xs text-muted-foreground truncate font-mono">
                              {project.path}
                            </div>
                          </button>
                          <div className="flex items-center gap-1 shrink-0">
                            <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                              {project.vcs}
                            </span>
                            <button
                              type="button"
                              onClick={(e) =>
                                handleRemoveRecent(e, project.path)
                              }
                              disabled={removeRecent.isPending}
                              className="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-destructive/10 hover:text-destructive transition-all"
                              title="Remove from recent"
                            >
                              <X className="h-3 w-3" />
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  </ScrollArea>
                </>
              ) : (
                <div className="text-center py-8 text-muted-foreground">
                  <FolderOpen className="h-10 w-10 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">No recent projects</p>
                  <Button
                    variant="link"
                    size="sm"
                    onClick={() => setMode("browse")}
                    className="mt-2"
                  >
                    Browse for a project
                  </Button>
                </div>
              )}
            </div>
          ) : (
            <div className="space-y-2">
              <div className="flex items-center gap-1 text-sm font-mono bg-muted p-2 rounded overflow-x-auto">
                <button
                  type="button"
                  onClick={() => setCurrentPath("/")}
                  className="hover:text-primary shrink-0"
                >
                  <Home className="h-4 w-4" />
                </button>
                {pathSegments.map((segment, index) => {
                  const path = "/" + pathSegments.slice(0, index + 1).join("/");
                  return (
                    <span key={path} className="flex items-center shrink-0">
                      <ChevronRight className="h-3 w-3 text-muted-foreground" />
                      <button
                        type="button"
                        onClick={() => setCurrentPath(path)}
                        className="hover:text-primary hover:underline"
                      >
                        {segment}
                      </button>
                    </span>
                  );
                })}
              </div>

              {browseData?.is_vcs_root && (
                <div className="flex items-center justify-between p-2 bg-green-500/10 border border-green-500/30 rounded">
                  <span className="text-sm text-green-600 dark:text-green-400">
                    This is a {browseData.vcs} repository
                  </span>
                  <Button
                    size="sm"
                    onClick={() => handleSelectFolder(browseData.current_path)}
                    disabled={isPending}
                  >
                    {isPending ? (
                      <Loader className="h-4 w-4" />
                    ) : (
                      "Open This Project"
                    )}
                  </Button>
                </div>
              )}

              {isBrowsing ? (
                <div className="flex items-center justify-center py-8">
                  <Loader />
                </div>
              ) : (
                <ScrollArea className="h-[280px]">
                  <div className="space-y-0.5 pr-4">
                    {browseData?.parent_path && (
                      <button
                        type="button"
                        onClick={() =>
                          handleBrowseFolder(browseData.parent_path!)
                        }
                        className="w-full flex items-center gap-2 p-2 rounded hover:bg-accent text-left text-muted-foreground"
                      >
                        <Folder className="h-4 w-4" />
                        <span className="text-sm">..</span>
                      </button>
                    )}
                    {browseData?.entries.map((entry) => (
                      <div
                        key={entry.path}
                        className={cn(
                          "flex items-center gap-2 p-2 rounded hover:bg-accent group",
                          entry.is_vcs_root && "bg-green-500/5",
                        )}
                      >
                        {entry.is_vcs_root ? (
                          <FolderGit className="h-4 w-4 text-green-600 dark:text-green-400 shrink-0" />
                        ) : (
                          <Folder className="h-4 w-4 text-muted-foreground shrink-0" />
                        )}
                        <button
                          type="button"
                          onClick={() =>
                            entry.is_vcs_root
                              ? handleSelectFolder(entry.path)
                              : handleBrowseFolder(entry.path)
                          }
                          disabled={isPending}
                          className="flex-1 text-left text-sm truncate disabled:opacity-50"
                        >
                          {entry.name}
                        </button>
                        {entry.is_vcs_root && (
                          <span className="text-xs px-1.5 py-0.5 rounded bg-green-500/20 text-green-600 dark:text-green-400 shrink-0">
                            {entry.vcs}
                          </span>
                        )}
                        {entry.is_vcs_root && (
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => handleSelectFolder(entry.path)}
                            disabled={isPending}
                            className="opacity-0 group-hover:opacity-100 shrink-0 h-6 px-2"
                          >
                            Open
                          </Button>
                        )}
                      </div>
                    ))}
                    {browseData?.entries.length === 0 && (
                      <div className="text-center py-4 text-muted-foreground text-sm">
                        No subdirectories
                      </div>
                    )}
                  </div>
                </ScrollArea>
              )}

              <div className="pt-2 border-t">
                <label
                  htmlFor="manual-path"
                  className="text-xs text-muted-foreground"
                >
                  Or paste a path:
                </label>
                <div className="flex gap-2 mt-1">
                  <Input
                    id="manual-path"
                    placeholder="/path/to/project"
                    className="font-mono text-sm"
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        const value = (
                          e.target as HTMLInputElement
                        ).value.trim();
                        if (value) handleSelectFolder(value);
                      }
                    }}
                  />
                </div>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>
    );
  },
);

export const ProjectPickerDialog = defineModal<
  ProjectPickerDialogProps,
  unknown
>(ProjectPickerDialogComponent);
