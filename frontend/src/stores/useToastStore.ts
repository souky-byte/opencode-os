import { create } from "zustand";

export type ToastType = "info" | "success" | "error";

export interface Toast {
	id: string;
	type: ToastType;
	message: string;
	duration?: number;
}

interface ToastState {
	toasts: Toast[];
	addToast: (toast: Omit<Toast, "id">) => void;
	removeToast: (id: string) => void;
}

let toastId = 0;

export const useToastStore = create<ToastState>()((set) => ({
	toasts: [],
	addToast: (toast) => {
		const id = `toast-${++toastId}`;
		const duration = toast.duration ?? 4000;

		set((state) => ({
			toasts: [...state.toasts, { ...toast, id }],
		}));

		// Auto-remove after duration
		setTimeout(() => {
			set((state) => ({
				toasts: state.toasts.filter((t) => t.id !== id),
			}));
		}, duration);
	},
	removeToast: (id) =>
		set((state) => ({
			toasts: state.toasts.filter((t) => t.id !== id),
		})),
}));

// Helper functions
export const toast = {
	info: (message: string, duration?: number) =>
		useToastStore.getState().addToast({ type: "info", message, duration }),
	success: (message: string, duration?: number) =>
		useToastStore.getState().addToast({ type: "success", message, duration }),
	error: (message: string, duration?: number) =>
		useToastStore.getState().addToast({ type: "error", message, duration }),
};
