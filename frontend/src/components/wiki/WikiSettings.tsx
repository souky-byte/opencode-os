import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useGetWikiSettings, useUpdateWikiSettings } from "@/api/generated/settings/settings";

export function WikiSettings() {
	const queryClient = useQueryClient();
	const { data: settingsData, isLoading } = useGetWikiSettings();
	const updateMutation = useUpdateWikiSettings();

	const [enabled, setEnabled] = useState(false);
	const [branches, setBranches] = useState<string[]>([]);
	const [branchInput, setBranchInput] = useState("");
	const [openrouterApiKey, setOpenrouterApiKey] = useState("");
	const [embeddingModel, setEmbeddingModel] = useState("");
	const [chatModel, setChatModel] = useState("");
	const [autoSync, setAutoSync] = useState(false);
	const [isDirty, setIsDirty] = useState(false);

	// Sync settings to local state
	useEffect(() => {
		if (settingsData?.status === 200) {
			const settings = settingsData.data;
			setEnabled(settings.enabled);
			setBranches(settings.branches);
			setEmbeddingModel(settings.embedding_model ?? "openai/text-embedding-3-small");
			setChatModel(settings.chat_model ?? "anthropic/claude-3.5-sonnet");
			setAutoSync(settings.auto_sync);
			// Don't set API key from response (it's masked)
		}
	}, [settingsData]);

	const handleSave = async () => {
		await updateMutation.mutateAsync({
			data: {
				enabled,
				branches,
				openrouter_api_key: openrouterApiKey || null,
				embedding_model: embeddingModel || null,
				chat_model: chatModel || null,
				auto_sync: autoSync,
			},
		});

		// Invalidate wiki queries to refresh status
		await queryClient.invalidateQueries({ queryKey: ["/api/wiki/status"] });
		setIsDirty(false);
		setOpenrouterApiKey(""); // Clear API key input after save
	};

	const handleAddBranch = () => {
		const branch = branchInput.trim();
		if (branch && !branches.includes(branch)) {
			setBranches([...branches, branch]);
			setBranchInput("");
			setIsDirty(true);
		}
	};

	const handleRemoveBranch = (branch: string) => {
		setBranches(branches.filter((b) => b !== branch));
		setIsDirty(true);
	};

	if (isLoading) {
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
					<label className="text-sm font-medium">Enable Wiki</label>
					<p className="text-xs text-muted-foreground">
						Enable wiki generation and semantic search
					</p>
				</div>
				<button
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
				<label className="text-sm font-medium">OpenRouter API Key</label>
				<input
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

			{/* Branches */}
			<div className="space-y-2">
				<label className="text-sm font-medium">Branches to Index</label>
				<div className="flex gap-2">
					<input
						type="text"
						value={branchInput}
						onChange={(e) => setBranchInput(e.target.value)}
						onKeyDown={(e) => {
							if (e.key === "Enter") {
								e.preventDefault();
								handleAddBranch();
							}
						}}
						placeholder="e.g., main, develop"
						className="flex-1 px-3 py-2 bg-accent border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm"
					/>
					<button
						type="button"
						onClick={handleAddBranch}
						className="px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 text-sm"
					>
						Add
					</button>
				</div>
				{branches.length > 0 && (
					<div className="flex flex-wrap gap-2 mt-2">
						{branches.map((branch) => (
							<span
								key={branch}
								className="inline-flex items-center gap-1 px-2 py-1 bg-accent rounded-md text-sm"
							>
								<span className="font-mono">{branch}</span>
								<button
									type="button"
									onClick={() => handleRemoveBranch(branch)}
									className="text-muted-foreground hover:text-foreground"
								>
									<svg
										className="h-3 w-3"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										strokeWidth="2"
									>
										<path d="M18 6L6 18M6 6l12 12" />
									</svg>
								</button>
							</span>
						))}
					</div>
				)}
			</div>

			{/* Models */}
			<div className="grid grid-cols-2 gap-4">
				<div className="space-y-2">
					<label className="text-sm font-medium">Embedding Model</label>
					<input
						type="text"
						value={embeddingModel}
						onChange={(e) => {
							setEmbeddingModel(e.target.value);
							setIsDirty(true);
						}}
						placeholder="openai/text-embedding-3-small"
						className="w-full px-3 py-2 bg-accent border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm font-mono"
					/>
				</div>
				<div className="space-y-2">
					<label className="text-sm font-medium">Chat Model</label>
					<input
						type="text"
						value={chatModel}
						onChange={(e) => {
							setChatModel(e.target.value);
							setIsDirty(true);
						}}
						placeholder="anthropic/claude-3.5-sonnet"
						className="w-full px-3 py-2 bg-accent border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm font-mono"
					/>
				</div>
			</div>

			{/* Auto-sync */}
			<div className="flex items-center justify-between">
				<div>
					<label className="text-sm font-medium">Auto-sync on Push</label>
					<p className="text-xs text-muted-foreground">
						Automatically re-index when code is pushed
					</p>
				</div>
				<button
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
