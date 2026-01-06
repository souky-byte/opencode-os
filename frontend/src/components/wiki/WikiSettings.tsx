import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useState } from "react";
import { useGetWikiSettings, useUpdateWikiSettings } from "@/api/generated/settings/settings";
import { useGetWikiStatus } from "@/api/generated/wiki/wiki";
import { useEventStream } from "@/hooks/useEventStream";
import { useWikiStore } from "@/stores/useWikiStore";
import { WikiIndexProgress } from "./WikiIndexProgress";

type WikiEvent = {
	type: "WikiIndexProgress";
	data: {
		branches: Array<{
			branch: string;
			state: string;
			// biome-ignore lint/style/useNamingConvention: Backend type
			file_count: number;
			// biome-ignore lint/style/useNamingConvention: Backend type
			chunk_count: number;
			// biome-ignore lint/style/useNamingConvention: Backend type
			last_indexed_at: string | null;
			// biome-ignore lint/style/useNamingConvention: Backend type
			progress_percent: number | null;
			// biome-ignore lint/style/useNamingConvention: Backend type
			error_message: string | null;
		}>;
		configured: boolean;
	};
};

type RemoteBranchesData = {
	// biome-ignore lint/style/useNamingConvention: Backend type
	remote_url: string | null;
	branches: string[];
	// biome-ignore lint/style/useNamingConvention: Backend type
	current_branch: string | null;
};

export function WikiSettings() {
	const queryClient = useQueryClient();
	const { setBranchStatuses, setIsIndexing, isIndexing: isStoreIndexing } = useWikiStore();
	const { data: settingsData, isLoading: isSettingsLoading } = useGetWikiSettings();
	const updateMutation = useUpdateWikiSettings();

	const [remoteBranches, setRemoteBranches] = useState<RemoteBranchesData | null>(null);
	const [isLoadingBranches, setIsLoadingBranches] = useState(false);

	useEventStream({
		onEvent: (event) => {
			const e = event as unknown as Event | WikiEvent;
			// biome-ignore lint/security/noSecrets: Event type
			if (e.type === "WikiIndexProgress") {
				const status = (e as WikiEvent).data;
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
		},
	});

	const { data: statusData } = useGetWikiStatus({
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
			setIsIndexing(
				status.branches.some((b) => b.state === "indexing" || b.state === "generating"),
			);
		}
	}, [statusData, setBranchStatuses, setIsIndexing]);

	const [enabled, setEnabled] = useState(false);
	const [branches, setBranches] = useState<string[]>([]);
	const [openrouterApiKey, setOpenrouterApiKey] = useState("");
	const [autoSync, setAutoSync] = useState(false);
	const [isDirty, setIsDirty] = useState(false);

	const fetchRemoteBranches = useCallback(async () => {
		setIsLoadingBranches(true);
		try {
			const response = await fetch("/api/wiki/remote-branches");
			if (response.ok) {
				const data = (await response.json()) as RemoteBranchesData;
				setRemoteBranches(data);
			}
		} catch (error) {
			console.error("Failed to fetch remote branches:", error);
		} finally {
			setIsLoadingBranches(false);
		}
	}, []);

	useEffect(() => {
		void fetchRemoteBranches();
	}, [fetchRemoteBranches]);

	useEffect(() => {
		if (settingsData?.status === 200) {
			const settings = settingsData.data;
			setEnabled(settings.enabled);
			setBranches(settings.branches);
			setAutoSync(settings.auto_sync);
		}
	}, [settingsData]);

	const handleSave = async () => {
		await updateMutation.mutateAsync({
			data: {
				enabled,
				branches,
				openrouter_api_key: openrouterApiKey || null,
				auto_sync: autoSync,
			},
		});

		await queryClient.invalidateQueries({ queryKey: ["/api/wiki/status"] });
		setIsDirty(false);
		setOpenrouterApiKey("");
	};

	const handleBranchToggle = (branch: string) => {
		if (branches.includes(branch)) {
			setBranches(branches.filter((b) => b !== branch));
		} else {
			setBranches([...branches, branch]);
		}
		setIsDirty(true);
	};

	if (isSettingsLoading) {
		return (
			<div className="rounded-lg border border-border p-4">
				<div className="animate-pulse space-y-3">
					<div className="h-4 bg-accent rounded w-1/4" />
					<div className="h-10 bg-accent rounded" />
					<div className="h-10 bg-accent rounded" />
				</div>
			</div>
		);
	}

	return (
		<div className="rounded-lg border border-border p-4 space-y-6">
			<div>
				<h3 className="font-medium">Wiki Settings</h3>
				<p className="text-sm text-muted-foreground mt-1">
					Configure AI-powered documentation generation and code search.
				</p>
			</div>

			{/* Enable/Disable */}
			<div className="flex items-center justify-between">
				<div>
					<label htmlFor="wiki-enable" className="text-sm font-medium">
						Enable Wiki
					</label>
					<p className="text-xs text-muted-foreground">
						Enable wiki generation and semantic search
					</p>
				</div>
				<button
					id="wiki-enable"
					type="button"
					role="switch"
					aria-checked={enabled}
					onClick={() => {
						setEnabled(!enabled);
						setIsDirty(true);
					}}
					className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
						enabled ? "bg-primary" : "bg-accent"
					}`}
				>
					<span
						className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
							enabled ? "translate-x-6" : "translate-x-1"
						}`}
					/>
				</button>
			</div>

			{/* OpenRouter API Key */}
			<div className="space-y-2">
				<label htmlFor="wiki-api-key" className="text-sm font-medium">
					OpenRouter API Key
				</label>
				<input
					id="wiki-api-key"
					type="password"
					value={openrouterApiKey}
					onChange={(e) => {
						setOpenrouterApiKey(e.target.value);
						setIsDirty(true);
					}}
					placeholder="sk-or-... (leave empty to keep current)"
					className="w-full px-3 py-2 bg-accent border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm"
				/>
				<p className="text-xs text-muted-foreground">
					Get your API key from{" "}
					<a
						href="https://openrouter.ai/keys"
						target="_blank"
						rel="noopener noreferrer"
						className="text-primary hover:underline"
					>
						openrouter.ai
					</a>
				</p>
			</div>

			{/* Remote Info */}
			{remoteBranches?.remote_url && (
				<div className="space-y-1 p-3 bg-accent/50 rounded-md">
					<div className="flex items-center gap-2 text-sm">
						<svg
							className="h-4 w-4 text-muted-foreground"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
						</svg>
						<span className="font-mono text-xs truncate">{remoteBranches.remote_url}</span>
					</div>
					{remoteBranches.current_branch && (
						<p className="text-xs text-muted-foreground">
							Current branch: <span className="font-mono">{remoteBranches.current_branch}</span>
						</p>
					)}
				</div>
			)}

			{/* Branches to Index */}
			<div className="space-y-2">
				<div className="flex items-center justify-between">
					<label className="text-sm font-medium">Branches to Index</label>
					<button
						type="button"
						onClick={() => void fetchRemoteBranches()}
						disabled={isLoadingBranches}
						className="text-xs text-primary hover:underline disabled:opacity-50"
					>
						{isLoadingBranches ? "Loading..." : "Refresh"}
					</button>
				</div>

				{isLoadingBranches ? (
					<div className="animate-pulse space-y-2">
						<div className="h-6 bg-accent rounded w-1/3" />
						<div className="h-6 bg-accent rounded w-1/4" />
					</div>
				) : remoteBranches && remoteBranches.branches.length > 0 ? (
					<div className="space-y-2 max-h-48 overflow-y-auto">
						{remoteBranches.branches.map((branch) => (
							<label
								key={branch}
								className="flex items-center gap-3 p-2 rounded-md hover:bg-accent/50 cursor-pointer"
							>
								<input
									type="checkbox"
									checked={branches.includes(branch)}
									onChange={() => handleBranchToggle(branch)}
									className="h-4 w-4 rounded border-border text-primary focus:ring-primary/50"
								/>
								<span className="font-mono text-sm">{branch}</span>
								{remoteBranches.current_branch === branch && (
									<span className="text-xs bg-primary/20 text-primary px-2 py-0.5 rounded">
										current
									</span>
								)}
							</label>
						))}
					</div>
				) : (
					<div className="text-sm text-muted-foreground p-3 bg-accent/30 rounded-md">
						{remoteBranches?.remote_url
							? "No remote branches found"
							: "No git remote configured for this project"}
					</div>
				)}

				{branches.length > 0 && (
					<p className="text-xs text-muted-foreground">
						{branches.length} branch{branches.length !== 1 ? "es" : ""} selected:{" "}
						<span className="font-mono">{branches.join(", ")}</span>
					</p>
				)}
			</div>

			{/* Indexing Status & Controls */}
			<div className="pt-2 border-t border-border">
				<WikiIndexProgress indexOnly />
			</div>

			{/* Auto-sync */}
			<div className="flex items-center justify-between">
				<div>
					<label htmlFor="wiki-autosync" className="text-sm font-medium">
						Auto-sync on Push
					</label>
					<p className="text-xs text-muted-foreground">
						Automatically re-index when code is pushed
					</p>
				</div>
				<button
					id="wiki-autosync"
					type="button"
					role="switch"
					aria-checked={autoSync}
					onClick={() => {
						setAutoSync(!autoSync);
						setIsDirty(true);
					}}
					className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
						autoSync ? "bg-primary" : "bg-accent"
					}`}
				>
					<span
						className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
							autoSync ? "translate-x-6" : "translate-x-1"
						}`}
					/>
				</button>
			</div>

			{/* Save button */}
			<div className="flex justify-end pt-2">
				<button
					type="button"
					onClick={handleSave}
					disabled={!isDirty || updateMutation.isPending}
					className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-sm transition-colors"
				>
					{updateMutation.isPending ? "Saving..." : "Save Settings"}
				</button>
			</div>

			{updateMutation.isSuccess && (
				<div className="text-sm text-green-500">Settings saved successfully!</div>
			)}

			{updateMutation.isError && (
				<div className="text-sm text-destructive">Failed to save settings. Please try again.</div>
			)}
		</div>
	);
}
