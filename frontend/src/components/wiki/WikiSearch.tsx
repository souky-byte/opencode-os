import { useState } from "react";
import { useSearchWiki } from "@/api/generated/wiki/wiki";
import { useWikiStore } from "@/stores/useWikiStore";

export function WikiSearch() {
	const { searchQuery, searchResults, setSearchQuery, setSearchResults, setIsSearching } =
		useWikiStore();
	const [localQuery, setLocalQuery] = useState(searchQuery);
	const [searchError, setSearchError] = useState<string | null>(null);

	const searchMutation = useSearchWiki();

	const handleSearch = async (e: React.FormEvent) => {
		e.preventDefault();
		if (!localQuery.trim()) return;

		setSearchQuery(localQuery);
		setIsSearching(true);
		setSearchError(null);

		try {
			const result = await searchMutation.mutateAsync({
				data: { query: localQuery, limit: 20 },
			});

			if (result.status === 200) {
				setSearchResults(
					result.data.results.map((r) => ({
						file_path: r.file_path,
						start_line: r.start_line,
						end_line: r.end_line,
						content: r.content,
						language: r.language ?? null,
						score: r.score,
					})),
				);
			} else {
				setSearchError("Search failed. Please try again.");
				setSearchResults([]);
			}
		} catch {
			setSearchError("Failed to connect to search service.");
			setSearchResults([]);
		} finally {
			setIsSearching(false);
		}
	};

	return (
		<div className="p-6 max-w-4xl mx-auto">
			<h2 className="text-xl font-semibold mb-4">Semantic Code Search</h2>
			<p className="text-sm text-muted-foreground mb-6">
				Search your codebase using natural language. The search uses AI embeddings to find
				semantically similar code.
			</p>

			{/* Search form */}
			<form onSubmit={handleSearch} className="mb-6">
				<div className="flex gap-2">
					<div className="relative flex-1">
						<input
							type="text"
							value={localQuery}
							onChange={(e) => setLocalQuery(e.target.value)}
							placeholder="Search code... (e.g., 'error handling in API routes')"
							className="w-full px-4 py-2 pl-10 bg-accent border border-border rounded-md focus:outline-none focus:ring-2 focus:ring-primary/50"
						/>
						<svg
							className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<circle cx="11" cy="11" r="8" />
							<path d="M21 21l-4.35-4.35" />
						</svg>
					</div>
					<button
						type="submit"
						disabled={searchMutation.isPending || !localQuery.trim()}
						className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						{searchMutation.isPending ? "Searching..." : "Search"}
					</button>
				</div>
			</form>

			{/* Results */}
			{searchError ? (
				<div className="text-center py-12">
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
					<h3 className="text-lg font-medium">Search Error</h3>
					<p className="mt-2 text-sm text-muted-foreground">{searchError}</p>
				</div>
			) : searchResults.length > 0 ? (
				<div className="space-y-4">
					<div className="text-sm text-muted-foreground">
						Found {searchResults.length} results for "{searchQuery}"
					</div>
					{searchResults.map((result, index) => (
						<SearchResultItem
							key={`${result.file_path}-${result.start_line}-${index}`}
							result={result}
						/>
					))}
				</div>
			) : searchQuery && !searchMutation.isPending ? (
				<div className="text-center py-12">
					<svg
						className="mx-auto h-12 w-12 text-muted-foreground"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="1.5"
					>
						<circle cx="11" cy="11" r="8" />
						<path d="M21 21l-4.35-4.35" />
					</svg>
					<h3 className="mt-4 text-lg font-medium">No results found</h3>
					<p className="mt-2 text-sm text-muted-foreground">
						Try a different search query or make sure your codebase is indexed.
					</p>
				</div>
			) : (
				<div className="text-center py-12 text-muted-foreground">
					<svg
						className="mx-auto h-12 w-12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="1.5"
					>
						<circle cx="11" cy="11" r="8" />
						<path d="M21 21l-4.35-4.35" />
					</svg>
					<p className="mt-4">Enter a search query to find code in your codebase</p>
				</div>
			)}
		</div>
	);
}

function SearchResultItem({
	result,
}: {
	result: {
		file_path: string;
		start_line: number;
		end_line: number;
		content: string;
		language: string | null;
		score: number;
	};
}) {
	const scorePercent = Math.round(result.score * 100);

	return (
		<div className="border border-border rounded-lg overflow-hidden bg-card">
			{/* Header */}
			<div className="flex items-center justify-between px-4 py-2 bg-accent/50 border-b border-border">
				<div className="flex items-center gap-2 min-w-0">
					<svg
						className="h-4 w-4 text-muted-foreground flex-shrink-0"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="1.5"
					>
						<path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
						<polyline points="14 2 14 8 20 8" />
					</svg>
					<span className="text-sm font-mono truncate">{result.file_path}</span>
					<span className="text-xs text-muted-foreground">
						:{result.start_line}-{result.end_line}
					</span>
				</div>
				<div className="flex items-center gap-2">
					{result.language && (
						<span className="text-xs px-2 py-0.5 rounded bg-primary/10 text-primary">
							{result.language}
						</span>
					)}
					<span className="text-xs text-muted-foreground">{scorePercent}% match</span>
				</div>
			</div>

			{/* Code content */}
			<pre className="p-4 overflow-x-auto text-sm">
				<code className="font-mono">{result.content}</code>
			</pre>
		</div>
	);
}
