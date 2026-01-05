import { useQueryClient } from "@tanstack/react-query";
import { useStartIndexing, getGetWikiStatusQueryKey } from "@/api/generated/wiki/wiki";
import { cn } from "@/lib/utils";
import { useWikiStore, type WikiGenerationProgress } from "@/stores/useWikiStore";

interface WikiIndexProgressProps {
	compact?: boolean;
	indexOnly?: boolean;
}

export function WikiIndexProgress({ compact = false, indexOnly = false }: WikiIndexProgressProps) {
	const queryClient = useQueryClient();
	const { branchStatuses, isIndexing, setIsIndexing, generationProgress } = useWikiStore();
	const startIndexingMutation = useStartIndexing();

	const handleStartIndexing = async () => {
		const branchToIndex = branchStatuses.find(
			(b) => b.state !== "indexed" && b.state !== "indexing",
		);

		setIsIndexing(true);

		try {
			if (branchToIndex) {
				await startIndexingMutation.mutateAsync({
					data: { branch: branchToIndex.branch, index_only: indexOnly },
				});
			} else {
				const firstBranch = branchStatuses[0];
				if (firstBranch) {
					await startIndexingMutation.mutateAsync({
						data: { branch: firstBranch.branch, force: true, index_only: indexOnly },
					});
				}
			}
			setTimeout(() => {
				void queryClient.invalidateQueries({ queryKey: getGetWikiStatusQueryKey() });
			}, 500);
		} catch {
			setIsIndexing(false);
		}
	};

	if (compact) {
		if (branchStatuses.length === 0 && !isIndexing && !generationProgress) {
			return null;
		}

		if (generationProgress) {
			return <CompactGenerationProgress progress={generationProgress} />;
		}

		const indexingBranch = branchStatuses.find(
			(b) => b.state === "indexing" || b.state === "generating",
		);
		const completedCount = branchStatuses.filter((b) => b.state === "indexed").length;
		const totalCount = branchStatuses.length;

		if (indexingBranch) {
			const label = indexingBranch.state === "generating" ? "Generating wiki" : "Indexing";
			return (
				<div className="flex items-center gap-2 text-xs text-muted-foreground">
					<div className="h-2 w-2 rounded-full bg-primary animate-pulse" />
					<span>
						{label} {indexingBranch.branch}...
					</span>
					{indexingBranch.progress_percent > 0 && (
						<span className="text-primary">{indexingBranch.progress_percent}%</span>
					)}
				</div>
			);
		}

		if (isIndexing) {
			return (
				<div className="flex items-center gap-2 text-xs text-muted-foreground">
					<div className="h-2 w-2 rounded-full bg-primary animate-pulse" />
					<span>Starting...</span>
				</div>
			);
		}

		return (
			<div className="flex items-center gap-2 text-xs text-muted-foreground">
				<div
					className={cn(
						"h-2 w-2 rounded-full",
						completedCount === totalCount && totalCount > 0 ? "bg-green-500" : "bg-yellow-500",
					)}
				/>
				<span>
					{completedCount}/{totalCount} branches indexed
				</span>
			</div>
		);
	}

	// Full mode - detailed status display
	return (
		<div className="space-y-4">
			<div className="flex items-center justify-between">
				<h3 className="text-sm font-medium">Indexing Status</h3>
				<button
					type="button"
					onClick={handleStartIndexing}
					disabled={isIndexing || startIndexingMutation.isPending}
					className="px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
				>
					{isIndexing
						? "Indexing..."
						: startIndexingMutation.isPending
							? "Starting..."
							: "Start Indexing"}
				</button>
			</div>

			{branchStatuses.length === 0 ? (
				<div className="text-sm text-muted-foreground">
					No branches configured. Add branches in Wiki settings to start indexing.
				</div>
			) : (
				<div className="space-y-3">
					{branchStatuses.map((branch) => (
						<BranchStatusItem key={branch.branch} branch={branch} />
					))}
				</div>
			)}
		</div>
	);
}

function BranchStatusItem({
	branch,
}: {
	branch: {
		branch: string;
		state: string;
		file_count: number;
		chunk_count: number;
		last_indexed_at: string | null;
		progress_percent: number;
		error_message: string | null;
	};
}) {
	const getStateColor = (state: string) => {
		switch (state) {
			case "indexed":
				return "text-green-500";
			case "indexing":
			case "generating":
				return "text-primary";
			case "failed":
				return "text-destructive";
			default:
				return "text-muted-foreground";
		}
	};

	const getStateLabel = (state: string) => {
		switch (state) {
			case "indexed":
				return "Indexed";
			case "indexing":
				return "Indexing...";
			case "generating":
				return "Generating wiki...";
			case "failed":
				return "Failed";
			case "not_indexed":
				return "Not Indexed";
			default:
				return state;
		}
	};

	return (
		<div className="border border-border rounded-lg p-3 bg-card">
			<div className="flex items-center justify-between mb-2">
				<div className="flex items-center gap-2">
					<svg
						className="h-4 w-4 text-muted-foreground"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="1.5"
					>
						<line x1="6" y1="3" x2="6" y2="15" />
						<circle cx="18" cy="6" r="3" />
						<circle cx="6" cy="18" r="3" />
						<path d="M18 9a9 9 0 01-9 9" />
					</svg>
					<span className="font-mono text-sm">{branch.branch}</span>
				</div>
				<span className={cn("text-xs font-medium", getStateColor(branch.state))}>
					{getStateLabel(branch.state)}
				</span>
			</div>

			{/* Progress bar for indexing/generating state */}
			{(branch.state === "indexing" || branch.state === "generating") && (
				<div className="mb-2">
					<div className="h-1.5 bg-accent rounded-full overflow-hidden">
						<div
							className="h-full bg-primary transition-all duration-300"
							style={{ width: `${branch.progress_percent}%` }}
						/>
					</div>
					<div className="mt-1 text-xs text-muted-foreground text-right">
						{branch.progress_percent}%
					</div>
				</div>
			)}

			{/* Stats */}
			<div className="flex gap-4 text-xs text-muted-foreground">
				<span>{branch.file_count} files</span>
				<span>{branch.chunk_count} chunks</span>
				{branch.last_indexed_at && (
					<span>Last indexed: {new Date(branch.last_indexed_at).toLocaleDateString()}</span>
				)}
			</div>

			{/* Error message */}
			{branch.error_message && (
				<div className="mt-2 text-xs text-destructive bg-destructive/10 px-2 py-1 rounded">
					{branch.error_message}
				</div>
			)}
		</div>
	);
}

function CompactGenerationProgress({ progress }: { progress: WikiGenerationProgress }) {
	const phaseLabels: Record<string, string> = {
		analyzing: "Analyzing",
		planning: "Planning",
		generating_pages: "Generating",
		completed: "Done",
		failed: "Failed",
	};

	const phaseLabel = phaseLabels[progress.phase] || progress.phase;
	const showPageProgress = progress.phase === "generating_pages" && progress.total > 0;

	return (
		<div className="flex items-center gap-2 text-xs text-muted-foreground">
			<div className="h-2 w-2 rounded-full bg-primary animate-pulse" />
			<span>
				{phaseLabel}
				{showPageProgress && (
					<span className="text-primary ml-1">
						{progress.current}/{progress.total}
					</span>
				)}
			</span>
			{progress.currentItem && showPageProgress && (
				<span className="hidden sm:inline text-foreground/70 truncate max-w-[150px]">
					{progress.currentItem}
				</span>
			)}
		</div>
	);
}
