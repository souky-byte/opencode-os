import { useToastStore } from "@/stores/useToastStore";
import { cn } from "@/lib/utils";

const TOAST_STYLES = {
	info: "bg-blue-500/10 border-blue-500/30 text-blue-400",
	success: "bg-green-500/10 border-green-500/30 text-green-400",
	error: "bg-red-500/10 border-red-500/30 text-red-400",
} as const;

const TOAST_ICONS = {
	info: (
		<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
			<circle cx="12" cy="12" r="10" />
			<line x1="12" y1="16" x2="12" y2="12" />
			<line x1="12" y1="8" x2="12.01" y2="8" />
		</svg>
	),
	success: (
		<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
			<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
			<polyline points="22 4 12 14.01 9 11.01" />
		</svg>
	),
	error: (
		<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
			<circle cx="12" cy="12" r="10" />
			<line x1="15" y1="9" x2="9" y2="15" />
			<line x1="9" y1="9" x2="15" y2="15" />
		</svg>
	),
} as const;

export function ToastContainer() {
	const toasts = useToastStore((s) => s.toasts);
	const removeToast = useToastStore((s) => s.removeToast);

	if (toasts.length === 0) return null;

	return (
		<div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
			{toasts.map((toast) => (
				<div
					key={toast.id}
					className={cn(
						"flex items-center gap-2 px-4 py-3 rounded-lg border backdrop-blur-sm",
						"shadow-lg pointer-events-auto animate-in slide-in-from-right-2 fade-in",
						"max-w-sm text-sm",
						TOAST_STYLES[toast.type],
					)}
				>
					{TOAST_ICONS[toast.type]}
					<span className="flex-1">{toast.message}</span>
					<button
						type="button"
						onClick={() => removeToast(toast.id)}
						className="opacity-60 hover:opacity-100 transition-opacity"
						aria-label="Dismiss"
					>
						<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
							<line x1="18" y1="6" x2="6" y2="18" />
							<line x1="6" y1="6" x2="18" y2="18" />
						</svg>
					</button>
				</div>
			))}
		</div>
	);
}
