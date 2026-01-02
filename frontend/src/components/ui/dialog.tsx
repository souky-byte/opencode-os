import type { MouseEvent, ReactNode } from "react";
import { createContext, useCallback, useContext, useState } from "react";
import { cn } from "@/lib/utils";

interface DialogContextValue {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

const DialogContext = createContext<DialogContextValue | null>(null);

function useDialogContext() {
	const context = useContext(DialogContext);
	if (!context) {
		throw new Error("Dialog components must be used within a Dialog");
	}
	return context;
}

interface DialogProps {
	children: ReactNode;
	open?: boolean;
	onOpenChange?: (open: boolean) => void;
}

function Dialog({
	children,
	open: controlledOpen,
	onOpenChange: controlledOnOpenChange,
}: DialogProps) {
	const [uncontrolledOpen, setUncontrolledOpen] = useState(false);
	const open = controlledOpen ?? uncontrolledOpen;
	const onOpenChange = controlledOnOpenChange ?? setUncontrolledOpen;

	return <DialogContext.Provider value={{ open, onOpenChange }}>{children}</DialogContext.Provider>;
}

interface DialogTriggerProps {
	children: ReactNode;
	asChild?: boolean;
}

function DialogTrigger({ children, asChild }: DialogTriggerProps) {
	const { onOpenChange } = useDialogContext();

	if (asChild) {
		return (
			<button type="button" onClick={() => onOpenChange(true)} className="contents">
				{children}
			</button>
		);
	}

	return (
		<button type="button" onClick={() => onOpenChange(true)}>
			{children}
		</button>
	);
}

interface DialogContentProps {
	children: ReactNode;
	className?: string;
	hideCloseButton?: boolean;
}

function DialogContent({ children, className, hideCloseButton }: DialogContentProps) {
	const { open, onOpenChange } = useDialogContext();

	const handleBackdropClick = useCallback(
		(e: MouseEvent) => {
			if (e.target === e.currentTarget) {
				onOpenChange(false);
			}
		},
		[onOpenChange],
	);

	if (!open) {
		return null;
	}

	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center p-4">
			{/* Backdrop */}
			<button
				type="button"
				className="fixed inset-0 bg-black/70 backdrop-blur-sm cursor-default animate-in fade-in-0"
				onClick={handleBackdropClick}
				onKeyDown={(e) => e.key === "Escape" && onOpenChange(false)}
				aria-label="Close dialog"
			/>
			{/* Content */}
			<div
				className={cn(
					"relative z-50 w-full max-w-lg",
					"border border-border bg-card rounded-xl",
					"p-6 shadow-2xl shadow-black/40",
					"animate-in fade-in-0 zoom-in-95 duration-200",
					className,
				)}
			>
				{children}
				{!hideCloseButton && (
					<button
						type="button"
						className="absolute right-4 top-4 rounded-lg p-1.5 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
						onClick={() => onOpenChange(false)}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							width="24"
							height="24"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
							strokeLinecap="round"
							strokeLinejoin="round"
							className="h-4 w-4"
							aria-hidden="true"
						>
							<title>Close</title>
							<path d="M18 6 6 18" />
							<path d="m6 6 12 12" />
						</svg>
						<span className="sr-only">Close</span>
					</button>
				)}
			</div>
		</div>
	);
}

interface DialogHeaderProps {
	children: ReactNode;
	className?: string;
}

function DialogHeader({ children, className }: DialogHeaderProps) {
	return (
		<div className={cn("flex flex-col space-y-1.5 text-center sm:text-left", className)}>
			{children}
		</div>
	);
}

interface DialogTitleProps {
	children: ReactNode;
	className?: string;
}

function DialogTitle({ children, className }: DialogTitleProps) {
	return (
		<h2 className={cn("text-lg font-semibold leading-none tracking-tight text-foreground", className)}>
			{children}
		</h2>
	);
}

interface DialogDescriptionProps {
	children: ReactNode;
	className?: string;
}

function DialogDescription({ children, className }: DialogDescriptionProps) {
	return <p className={cn("text-sm text-muted-foreground", className)}>{children}</p>;
}

interface DialogFooterProps {
	children: ReactNode;
	className?: string;
}

function DialogFooter({ children, className }: DialogFooterProps) {
	return (
		<div className={cn("flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2", className)}>
			{children}
		</div>
	);
}

export {
	Dialog,
	DialogTrigger,
	DialogContent,
	DialogHeader,
	DialogTitle,
	DialogDescription,
	DialogFooter,
	useDialogContext,
};
