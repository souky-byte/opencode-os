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
	reset: () => void;
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
	reset: () => set({ currentProject: null, isLoading: false, error: null }),
}));

export const useCurrentProject = () => useProjectStore((s) => s.currentProject);
export const useProjectLoading = () => useProjectStore((s) => s.isLoading);
export const useProjectError = () => useProjectStore((s) => s.error);
export const useProjectDialogOpen = () => useProjectStore((s) => s.isDialogOpen);
