import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import {
	getGetWikiStatusQueryKey,
	useGenerateWiki,
	useGetWikiStatus,
	useGetWikiStructure,
} from "@/api/generated/wiki/wiki";
import { Loader } from "@/components/ui/loader";
import { useEventStream } from "@/hooks/useEventStream";
import {
	useWikiStore,
	type WikiGenerationPhase,
	type WikiGenerationProgress,
} from "@/stores/useWikiStore";
import { WikiChat } from "./WikiChat";
import { WikiIndexProgress } from "./WikiIndexProgress";
import { WikiPage } from "./WikiPage";
import { WikiSearch } from "./WikiSearch";
import { WikiSidebar } from "./WikiSidebar";

type WikiGenerationProgressEvent = {
	type: "wiki.generation_progress";
	branch: string;
	phase: WikiGenerationPhase;
	current: number;
	total: number;
	// biome-ignore lint/style/useNamingConvention: Backend event field
	current_item: string | null;
	message: string | null;
};

export function WikiView() {
	const queryClient = useQueryClient();
	const {
		viewMode,
		setStructure,
		setSections,
		setBranchStatuses,
		setIsIndexing,
		setGenerationProgress,
		isIndexing: isStoreIndexing,
		generationProgress,
	} = useWikiStore();
	const hasLoadedOnce = useRef(false);
	const generateWikiMutation = useGenerateWiki();

	useEventStream({
		onEvent: (event) => {
			if (event.type === "wiki.generation_progress") {
				const e = event as unknown as WikiGenerationProgressEvent;
				setGenerationProgress({
					branch: e.branch,
					phase: e.phase,
					current: e.current,
					total: e.total,
					currentItem: e.current_item ?? null,
					message: e.message ?? null,
				});

				const isFinished = e.phase === "completed" || e.phase === "failed";
				if (isFinished) {
					setIsIndexing(false);
					setGenerationProgress(null);
					void queryClient.invalidateQueries({ queryKey: getGetWikiStatusQueryKey() });
					void queryClient.invalidateQueries({ queryKey: ["/api/wiki/structure"] });
				} else {
					setIsIndexing(true);
				}
			}
		},
	});

	const {
		data: statusData,
		isLoading: isStatusLoading,
		isError: isStatusError,
	} = useGetWikiStatus({
		query: {
			refetchInterval: (query) => {
				const branches = query.state.data?.data?.branches;
				const isProcessingFromQuery = branches?.some(
					(b) => b.state === "indexing" || b.state === "generating",
				);
				if (isProcessingFromQuery || isStoreIndexing) {
					return 2000;
				}
				return 30000;
			},
		},
	});

	const isProcessing = statusData?.data?.branches?.some(
		(b) => b.state === "indexing" || b.state === "generating",
	);

	const { data: structureData, isLoading: isStructureLoading } = useGetWikiStructure(undefined, {
		query: {
			refetchInterval: isProcessing ? 5000 : false,
		},
	});

	useEffect(() => {
		if (statusData?.status === 200) {
			hasLoadedOnce.current = true;
			const status = statusData.data;
			setBranchStatuses(
				status.branches.map((b) => ({
					branch: b.branch,
					state: b.state,
					file_count: b.file_count,
					chunk_count: b.chunk_count,
					last_indexed_at: b.last_indexed_at ?? null,
					progress_percent: b.progress_percent ?? 0,
					error_message: b.error_message ?? null,
				})),
			);
			setIsIndexing(
				status.branches.some((b) => b.state === "indexing" || b.state === "generating"),
			);
		}
	}, [statusData, setBranchStatuses, setIsIndexing]);

	useEffect(() => {
		if (structureData?.status === 200 && structureData.data.root) {
			setStructure(structureData.data.root);
			setSections(
				(structureData.data.sections || []).map((s) => ({
					id: s.id,
					title: s.title,
					description: s.description ?? null,
					page_slugs: s.page_slugs,
					order: s.order,
				})),
			);
		}
	}, [structureData, setStructure, setSections]);

	const isInitialLoading = (isStatusLoading || isStructureLoading) && !hasLoadedOnce.current;
	const isConfigured = statusData?.status === 200 && statusData.data.configured;

	const isIndexingFromApi = statusData?.data?.branches?.some((b) => b.state === "indexing");
	const isGeneratingFromApi = statusData?.data?.branches?.some((b) => b.state === "generating");

	const hasIndexedContent =
		statusData?.status === 200 &&
		statusData.data.branches.some(
			(b) => (b.state === "indexed" || b.state === "generating") && b.chunk_count > 0,
		);
	const hasWikiPages =
		statusData?.status === 200 &&
		statusData.data.branches.some((b) => b.state === "indexed" && b.page_count > 0);
	const hasEmbeddingsOnly = hasIndexedContent && !hasWikiPages;

	const handleGenerateWiki = async () => {
		const branchToGenerate = statusData?.data?.branches?.find(
			(b) => b.state === "indexed" && b.chunk_count > 0 && b.page_count === 0,
		);
		if (branchToGenerate) {
			setIsIndexing(true);
			try {
				await generateWikiMutation.mutateAsync({
					data: { branch: branchToGenerate.branch },
				});
				setTimeout(() => {
					void queryClient.invalidateQueries({ queryKey: getGetWikiStatusQueryKey() });
				}, 500);
			} catch {
				setIsIndexing(false);
			}
		}
	};

	if (isInitialLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<Loader />
			</div>
		);
	}

	if (isStatusError) {
		return (
			<div className="flex h-full items-center justify-center p-6">
				<div className="max-w-md text-center">
					<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-destructive/10">
						<svg
							className="h-6 w-6 text-destructive"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="1.5"
							aria-hidden="true"
						>
							<circle cx="12" cy="12" r="10" />
							<line x1="12" y1="8" x2="12" y2="12" />
							<line x1="12" y1="16" x2="12.01" y2="16" />
						</svg>
					</div>
					<h2 className="text-xl font-semibold">Failed to Load Wiki</h2>
					<p className="mt-2 text-muted-foreground">
						Could not connect to the wiki service. Please check that the server is running.
					</p>
				</div>
			</div>
		);
	}

	if (!isConfigured) {
		return (
			<div className="flex h-full items-center justify-center p-6">
				<div className="max-w-md text-center">
					<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
						<svg
							className="h-6 w-6 text-primary"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="1.5"
							aria-hidden="true"
						>
							<path d="M4 19.5A2.5 2.5 0 016.5 17H20" />
							<path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z" />
							<path d="M8 7h8" />
							<path d="M8 11h8" />
							<path d="M8 15h5" />
						</svg>
					</div>
					<h2 className="text-xl font-semibold">Wiki Not Configured</h2>
					<p className="mt-2 text-muted-foreground">
						Configure your Wiki settings to enable AI-powered documentation and code search.
					</p>
					<p className="mt-4 text-sm text-muted-foreground">
						Add your OpenRouter API key in the Wiki settings to get started.
					</p>
				</div>
			</div>
		);
	}

	return (
		<div className="flex h-full">
			{/* Sidebar with tree navigation */}
			<WikiSidebar />

			{/* Main content area */}
			<div className="flex flex-1 flex-col overflow-hidden">
				{/* Top bar with view mode tabs */}
				<div className="flex items-center gap-4 border-b border-border px-4 py-2">
					<ViewModeTab mode="page" label="Documentation" />
					<ViewModeTab mode="search" label="Search" />
					<ViewModeTab mode="chat" label="Ask" />
					<div className="flex-1" />
					<WikiIndexProgress compact />
				</div>

				{/* Content based on view mode */}
				<div className="flex-1 overflow-auto">
					{hasWikiPages ? (
						<>
							{viewMode === "page" && <WikiPage />}
							{viewMode === "search" && <WikiSearch />}
							{viewMode === "chat" && <WikiChat />}
						</>
					) : hasEmbeddingsOnly || isGeneratingFromApi ? (
						<div className="flex h-full items-center justify-center p-6">
							<div className="max-w-md text-center">
								<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
									{isGeneratingFromApi ? (
										<svg
											className="h-6 w-6 text-primary animate-spin"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											strokeWidth="2"
											aria-hidden="true"
										>
											<path d="M21 12a9 9 0 11-6.219-8.56" />
										</svg>
									) : (
										<svg
											className="h-6 w-6 text-primary"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											strokeWidth="1.5"
											aria-hidden="true"
										>
											<path d="M9 12h6M12 9v6" />
											<path d="M4 19.5A2.5 2.5 0 016.5 17H20" />
											<path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z" />
										</svg>
									)}
								</div>
								<h2 className="text-xl font-semibold">
									{isGeneratingFromApi ? "Generating Wiki..." : "Codebase Indexed"}
								</h2>
								<p className="mt-2 text-muted-foreground">
									{isGeneratingFromApi
										? "Wiki pages are being generated. This may take a few minutes."
										: "Your codebase has been indexed. Generate wiki documentation to view it here."}
								</p>
								{!isGeneratingFromApi && (
									<div className="mt-6">
										<button
											type="button"
											onClick={handleGenerateWiki}
											disabled={generateWikiMutation.isPending}
											className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
										>
											{generateWikiMutation.isPending ? "Starting..." : "Generate Wiki"}
										</button>
									</div>
								)}
								{isGeneratingFromApi && (
									<div className="mt-6">
										<WikiGenerationProgressDisplay progress={generationProgress} />
									</div>
								)}
							</div>
						</div>
					) : isIndexingFromApi || isStoreIndexing ? (
						<div className="flex h-full items-center justify-center p-6">
							<div className="max-w-md text-center">
								<div className="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
									<svg
										className="h-6 w-6 text-primary animate-spin"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										strokeWidth="2"
										aria-hidden="true"
									>
										<path d="M21 12a9 9 0 11-6.219-8.56" />
									</svg>
								</div>
								<h2 className="text-xl font-semibold">Indexing Codebase...</h2>
								<p className="mt-2 text-muted-foreground">
									Creating embeddings for your code. This may take a few minutes.
								</p>
								<div className="mt-6">
									<WikiIndexProgress compact />
								</div>
							</div>
						</div>
					) : (
						<div className="flex h-full items-center justify-center p-6">
							<div className="max-w-md text-center">
								<h2 className="text-xl font-semibold">No Content Indexed</h2>
								<p className="mt-2 text-muted-foreground">
									Your codebase is not indexed. Go to Settings to start indexing.
								</p>
								<div className="mt-6">
									<Link to="/settings">
										<button
											type="button"
											className="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground hover:bg-primary/90 h-10 px-4 py-2"
										>
											Go to Settings
										</button>
									</Link>
								</div>
							</div>
						</div>
					)}
				</div>
			</div>
		</div>
	);
}

function ViewModeTab({ mode, label }: { mode: "page" | "search" | "chat"; label: string }) {
	const { viewMode, setViewMode } = useWikiStore();
	const isActive = viewMode === mode;

	return (
		<button
			type="button"
			onClick={() => setViewMode(mode)}
			className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
				isActive
					? "bg-primary/10 text-primary"
					: "text-muted-foreground hover:bg-accent hover:text-foreground"
			}`}
		>
			{label}
		</button>
	);
}

function WikiGenerationProgressDisplay({ progress }: { progress: WikiGenerationProgress | null }) {
	if (!progress) {
		return (
			<div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
				<div className="h-2 w-2 rounded-full bg-primary animate-pulse" />
				<span>Starting generation...</span>
			</div>
		);
	}

	const phaseLabels: Record<string, string> = {
		analyzing: "Analyzing project structure",
		planning: "Planning wiki structure",
		generating_pages: "Generating pages",
		completed: "Completed",
		failed: "Failed",
	};

	const phaseLabel = phaseLabels[progress.phase] || progress.phase;
	const showProgress = progress.phase === "generating_pages" && progress.total > 0;

	return (
		<div className="space-y-3">
			<div className="flex items-center justify-center gap-2">
				<PhaseIndicator phase={progress.phase} />
			</div>

			<div className="text-sm font-medium text-foreground">{phaseLabel}</div>

			{showProgress && (
				<>
					<div className="w-full h-2 bg-accent rounded-full overflow-hidden">
						<div
							className="h-full bg-primary transition-all duration-300 ease-out"
							style={{ width: `${(progress.current / progress.total) * 100}%` }}
						/>
					</div>
					<div className="text-xs text-muted-foreground">
						Page {progress.current} of {progress.total}
						{progress.currentItem && (
							<span className="block mt-1 text-foreground truncate max-w-[300px]">
								{progress.currentItem}
							</span>
						)}
					</div>
				</>
			)}

			{progress.message && !showProgress && (
				<div className="text-xs text-muted-foreground">{progress.message}</div>
			)}
		</div>
	);
}

function PhaseIndicator({ phase }: { phase: string }) {
	const phases = ["analyzing", "planning", "generating_pages"];
	const currentIndex = phases.indexOf(phase);

	return (
		<div className="flex items-center gap-1">
			{phases.map((p, index) => {
				const isActive = p === phase;
				const isCompleted = index < currentIndex;

				return (
					<div key={p} className="flex items-center">
						<div
							className={`h-2 w-2 rounded-full transition-colors ${
								isActive ? "bg-primary animate-pulse" : isCompleted ? "bg-primary" : "bg-muted"
							}`}
						/>
						{index < phases.length - 1 && (
							<div
								className={`h-0.5 w-4 transition-colors ${isCompleted ? "bg-primary" : "bg-muted"}`}
							/>
						)}
					</div>
				);
			})}
		</div>
	);
}
