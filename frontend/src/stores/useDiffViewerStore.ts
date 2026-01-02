import { create } from "zustand";

type State = {
  isExpanded: boolean;
  collapsedFiles: Set<string>;
  setExpanded: (expanded: boolean) => void;
  toggleFileCollapsed: (filePath: string) => void;
  isFileCollapsed: (filePath: string) => boolean;
};

export const useDiffViewerStore = create<State>()((set, get) => ({
  isExpanded: false,
  collapsedFiles: new Set<string>(),

  setExpanded: (expanded) => set({ isExpanded: expanded }),

  toggleFileCollapsed: (filePath) =>
    set((state) => {
      const newCollapsed = new Set(state.collapsedFiles);
      if (newCollapsed.has(filePath)) {
        newCollapsed.delete(filePath);
      } else {
        newCollapsed.add(filePath);
      }
      return { collapsedFiles: newCollapsed };
    }),

  isFileCollapsed: (filePath) => get().collapsedFiles.has(filePath),
}));
