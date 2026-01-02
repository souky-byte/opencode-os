import { create } from "zustand";
import { persist } from "zustand/middleware";

export type SidebarView = "kanban" | "sessions" | "settings";

type State = {
	collapsed: boolean;
	activeView: SidebarView;
	setCollapsed: (collapsed: boolean) => void;
	toggleCollapsed: () => void;
	setActiveView: (view: SidebarView) => void;
};

export const useSidebarStore = create<State>()(
	persist(
		(set) => ({
			collapsed: false,
			activeView: "kanban",
			setCollapsed: (collapsed) => set({ collapsed }),
			toggleCollapsed: () => set((s) => ({ collapsed: !s.collapsed })),
			setActiveView: (activeView) => set({ activeView }),
		}),
		{
			name: "sidebar-store",
			partialize: (state) => ({
				collapsed: state.collapsed,
				activeView: state.activeView,
			}),
		},
	),
);

export const useSidebarCollapsed = () => useSidebarStore((s) => s.collapsed);
export const useSidebarActiveView = () => useSidebarStore((s) => s.activeView);
