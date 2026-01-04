import { create } from "zustand";

export type WikiViewMode = "page" | "search" | "chat";

export type WikiTreeNode = {
	slug: string;
	title: string;
	page_type: string;
	order: number;
	children: WikiTreeNode[];
};

export type WikiPage = {
	slug: string;
	title: string;
	content: string;
	page_type: string;
	parent_slug: string | null;
	file_paths: string[];
	has_diagrams: boolean;
	updated_at: string;
};

export type WikiSearchResult = {
	file_path: string;
	start_line: number;
	end_line: number;
	content: string;
	language: string | null;
	score: number;
};

export type BranchStatus = {
	branch: string;
	state: string;
	file_count: number;
	chunk_count: number;
	last_indexed_at: string | null;
	progress_percent: number;
	error_message: string | null;
};

export type ChatMessage = {
	role: "user" | "assistant";
	content: string;
	sources?: Array<{
		file_path: string;
		start_line: number;
		end_line: number;
		score: number;
		snippet: string;
	}>;
};

type State = {
	viewMode: WikiViewMode;
	currentPageSlug: string | null;
	structure: WikiTreeNode | null;
	searchQuery: string;
	searchResults: WikiSearchResult[];
	isSearching: boolean;
	chatMessages: ChatMessage[];
	chatInput: string;
	isChatLoading: boolean;
	conversationId: string | null;
	branchStatuses: BranchStatus[];
	isIndexing: boolean;

	setViewMode: (mode: WikiViewMode) => void;
	setCurrentPageSlug: (slug: string | null) => void;
	setStructure: (structure: WikiTreeNode | null) => void;
	setSearchQuery: (query: string) => void;
	setSearchResults: (results: WikiSearchResult[]) => void;
	setIsSearching: (isSearching: boolean) => void;
	addChatMessage: (message: ChatMessage) => void;
	setChatInput: (input: string) => void;
	setIsChatLoading: (isLoading: boolean) => void;
	setConversationId: (id: string | null) => void;
	clearChat: () => void;
	setBranchStatuses: (statuses: BranchStatus[]) => void;
	setIsIndexing: (isIndexing: boolean) => void;
	reset: () => void;
};

const initialState = {
	viewMode: "page" as WikiViewMode,
	currentPageSlug: null,
	structure: null,
	searchQuery: "",
	searchResults: [],
	isSearching: false,
	chatMessages: [],
	chatInput: "",
	isChatLoading: false,
	conversationId: null,
	branchStatuses: [],
	isIndexing: false,
};

export const useWikiStore = create<State>((set) => ({
	...initialState,

	setViewMode: (viewMode) => set({ viewMode }),
	setCurrentPageSlug: (currentPageSlug) => set({ currentPageSlug }),
	setStructure: (structure) => set({ structure }),
	setSearchQuery: (searchQuery) => set({ searchQuery }),
	setSearchResults: (searchResults) => set({ searchResults }),
	setIsSearching: (isSearching) => set({ isSearching }),
	addChatMessage: (message) => set((state) => ({ chatMessages: [...state.chatMessages, message] })),
	setChatInput: (chatInput) => set({ chatInput }),
	setIsChatLoading: (isChatLoading) => set({ isChatLoading }),
	setConversationId: (conversationId) => set({ conversationId }),
	clearChat: () => set({ chatMessages: [], conversationId: null }),
	setBranchStatuses: (branchStatuses) => set({ branchStatuses }),
	setIsIndexing: (isIndexing) => set({ isIndexing }),
	reset: () => set(initialState),
}));
