import { useEffect } from "react";
import { useGetWikiStatus, useGetWikiStructure } from "@/api/generated/wiki/wiki";
import { Loader } from "@/components/ui/loader";
import { useWikiStore } from "@/stores/useWikiStore";
import { WikiChat } from "./WikiChat";
import { WikiIndexProgress } from "./WikiIndexProgress";
import { WikiPage } from "./WikiPage";
import { WikiSearch } from "./WikiSearch";
import { WikiSidebar } from "./WikiSidebar";

export function WikiView() {
	const { viewMode, setStructure, setBranchStatuses, setIsIndexing } = useWikiStore();

	const {
		data: statusData,
		isLoading: isStatusLoading,
		isError: isStatusError,
	} = useGetWikiStatus({
		query: {
			refetchInterval: 5000,
		},
	});

	const { data: structureData, isLoading: isStructureLoading } = useGetWikiStructure();

	// Sync status data to store
	useEffect(() => {
		if (statusData?.status === 200) {
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
			setIsIndexing(status.branches.some((b) => b.state === "indexing"));
		}
	}, [statusData, setBranchStatuses, setIsIndexing]);

	useEffect(() => {
		if (structureData?.status === 200 && structureData.data.root) {
			setStructure(structureData.data.root);
		}
	}, [structureData, setStructure]);

	const isLoading = isStatusLoading || isStructureLoading;
	const isConfigured = statusData?.status === 200 && statusData.data.configured;
	const hasIndexedContent =
		statusData?.status === 200 &&
		statusData.data.branches.some((b) => b.state === "indexed" && b.chunk_count > 0);

	if (isLoading) {
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
					{hasIndexedContent ? (
						<>
							{viewMode === "page" && <WikiPage />}
							{viewMode === "search" && <WikiSearch />}
							{viewMode === "chat" && <WikiChat />}
						</>
					) : (
						<div className="flex h-full items-center justify-center p-6">
							<div className="max-w-md text-center">
								<h2 className="text-xl font-semibold">No Content Indexed</h2>
								<p className="mt-2 text-muted-foreground">
									Start indexing your codebase to generate documentation and enable search.
								</p>
								<div className="mt-6">
									<WikiIndexProgress />
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
