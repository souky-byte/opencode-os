import { useEffect } from "react";
import { ArrowLeft } from "lucide-react";
import { useGetWorkspaceDiff } from "@/api/generated/workspaces/workspaces";
import { useGetViewedFiles, useSetFileViewed } from "@/api/generated/workspaces/workspaces";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Loader } from "@/components/ui/loader";
import { useDiffViewerStore } from "@/stores/useDiffViewerStore";
import { DiffFileSection } from "./DiffFileSection";
import { useDiffParser, getFileDisplayName } from "./useDiffParser";

interface DiffViewerProps {
	taskId: string;
	onClose?: () => void;
}

export function DiffViewer({ taskId, onClose }: DiffViewerProps) {
	const { setExpanded } = useDiffViewerStore();

	// Set expanded mode on mount
	useEffect(() => {
		setExpanded(true);
		return () => setExpanded(false);
	}, [setExpanded]);

	// Fetch diff data
	const { data: diffData, isLoading: isDiffLoading } = useGetWorkspaceDiff(taskId, {
		query: {
			staleTime: 10000,
		},
	});

	// Fetch viewed state
	const { data: viewedData } = useGetViewedFiles(taskId, {
		query: {
			staleTime: 5000,
		},
	});

	// Mutation for setting viewed state
	const { mutate: setViewed } = useSetFileViewed();

	// Parse diff
	const parsedDiff = useDiffParser(diffData?.data?.diff);
	const viewedFiles = viewedData?.data?.viewed_files ?? [];

	const handleToggleViewed = (filePath: string, viewed: boolean) => {
		setViewed({
			taskId,
			data: { file_path: filePath, viewed },
		});
	};

	const viewedCount = parsedDiff?.files.filter((f) =>
		viewedFiles.includes(getFileDisplayName(f))
	).length ?? 0;

	if (isDiffLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<Loader />
			</div>
		);
	}

	if (!parsedDiff || parsedDiff.files.length === 0) {
		return (
			<div className="flex flex-col h-full">
				<DiffViewerHeader
					onClose={onClose}
					viewedCount={0}
					totalCount={0}
					totalAdditions={0}
					totalDeletions={0}
				/>
				<div className="flex-1 flex items-center justify-center">
					<div className="text-center text-muted-foreground">
						<p>No changes to display</p>
						<p className="text-sm mt-1">Make some changes in the workspace first</p>
					</div>
				</div>
			</div>
		);
	}

	return (
		<div className="flex flex-col h-full">
			<DiffViewerHeader
				onClose={onClose}
				viewedCount={viewedCount}
				totalCount={parsedDiff.fileCount}
				totalAdditions={parsedDiff.totalAdditions}
				totalDeletions={parsedDiff.totalDeletions}
			/>

			<ScrollArea className="flex-1">
				<div className="divide-y divide-border">
					{parsedDiff.files.map((file) => {
						const filePath = getFileDisplayName(file);
						return (
							<DiffFileSection
								key={filePath}
								file={file}
								isViewed={viewedFiles.includes(filePath)}
								onToggleViewed={handleToggleViewed}
							/>
						);
					})}
				</div>
			</ScrollArea>
		</div>
	);
}

interface DiffViewerHeaderProps {
	onClose?: () => void;
	viewedCount: number;
	totalCount: number;
	totalAdditions: number;
	totalDeletions: number;
}

function DiffViewerHeader({
	onClose,
	viewedCount,
	totalCount,
	totalAdditions,
	totalDeletions,
}: DiffViewerHeaderProps) {
	return (
		<div className="flex items-center gap-4 px-4 py-3 border-b border-border bg-card/50 shrink-0">
			{onClose && (
				<button
					type="button"
					onClick={onClose}
					className="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors"
				>
					<ArrowLeft className="h-4 w-4" />
					Back
				</button>
			)}

			<div className="flex-1" />

			<div className="flex items-center gap-4 text-sm">
				<span className="text-muted-foreground">
					Files: {viewedCount}/{totalCount} viewed
				</span>
				<div className="flex items-center gap-2 font-mono text-xs">
					<span className="text-green-500">+{totalAdditions}</span>
					<span className="text-red-500">-{totalDeletions}</span>
				</div>
			</div>
		</div>
	);
}
