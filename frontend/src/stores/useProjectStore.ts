import { create } from "zustand";
import type { ProjectInfo } from "../api/generated/model/projectInfo";

type ProjectState = {
  currentProject: ProjectInfo | null;
  isLoading: boolean;
  error: string | null;
  isDialogOpen: boolean;
  setCurrentProject: (project: ProjectInfo | null) => void;
  setLoading: (isLoading: boolean) => void;
  setError: (error: string | null) => void;
  openDialog: () => void;
  closeDialog: () => void;
};

export const useProjectStore = create<ProjectState>()((set) => ({
  currentProject: null,
  isLoading: true,
  error: null,
  isDialogOpen: false,
  setCurrentProject: (currentProject) => set({ currentProject, error: null }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error, isLoading: false }),
  openDialog: () => set({ isDialogOpen: true }),
  closeDialog: () => set({ isDialogOpen: false }),
}));
