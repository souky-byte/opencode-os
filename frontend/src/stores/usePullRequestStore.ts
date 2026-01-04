import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { PrState } from "../api/generated/model/prState";

export type PrSortField = "updated" | "created";
export type PrSortOrder = "asc" | "desc";

interface PrFilters {
  state: PrState | "all";
  author: string | null;
  label: string | null;
}

interface PrSort {
  field: PrSortField;
  order: PrSortOrder;
}

interface PullRequestStoreState {
  // Selected PR
  selectedPrNumber: number | null;

  // Filters
  filters: PrFilters;

  // Sorting
  sort: PrSort;

  // Selected comments for AI fix
  selectedCommentIds: number[];

  // Active tab in detail view
  activeTab: "conversation" | "files" | "diff";

  // Actions
  selectPr: (number: number | null) => void;
  setFilter: <K extends keyof PrFilters>(key: K, value: PrFilters[K]) => void;
  resetFilters: () => void;
  setSort: (field: PrSortField, order: PrSortOrder) => void;
  toggleCommentSelection: (commentId: number) => void;
  selectAllComments: (commentIds: number[]) => void;
  clearCommentSelection: () => void;
  setActiveTab: (tab: "conversation" | "files" | "diff") => void;
}

const defaultFilters: PrFilters = {
  state: "open",
  author: null,
  label: null,
};

const defaultSort: PrSort = {
  field: "updated",
  order: "desc",
};

export const usePullRequestStore = create<PullRequestStoreState>()(
  persist(
    (set) => ({
      selectedPrNumber: null,
      filters: defaultFilters,
      sort: defaultSort,
      selectedCommentIds: [],
      activeTab: "conversation",

      selectPr: (number) =>
        set({
          selectedPrNumber: number,
          selectedCommentIds: [],
          activeTab: "conversation",
        }),

      setFilter: (key, value) =>
        set((state) => ({
          filters: { ...state.filters, [key]: value },
        })),

      resetFilters: () => set({ filters: defaultFilters }),

      setSort: (field, order) => set({ sort: { field, order } }),

      toggleCommentSelection: (commentId) =>
        set((state) => {
          const isSelected = state.selectedCommentIds.includes(commentId);
          return {
            selectedCommentIds: isSelected
              ? state.selectedCommentIds.filter((id) => id !== commentId)
              : [...state.selectedCommentIds, commentId],
          };
        }),

      selectAllComments: (commentIds) => set({ selectedCommentIds: commentIds }),

      clearCommentSelection: () => set({ selectedCommentIds: [] }),

      setActiveTab: (tab) => set({ activeTab: tab }),
    }),
    {
      name: "pull-request-store",
      partialize: (state) => ({
        filters: state.filters,
        sort: state.sort,
      }),
    },
  ),
);
