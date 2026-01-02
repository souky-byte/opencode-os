import {
	createContext,
	useContext,
	useState,
	type ReactNode,
	useCallback,
} from "react";
import { cn } from "@/lib/utils";
import { Icon } from "./icon";

interface CollapsibleContextValue {
	open: boolean;
	onOpenChange: (open: boolean) => void;
}

const CollapsibleContext = createContext<CollapsibleContextValue | null>(null);

function useCollapsible() {
	const context = useContext(CollapsibleContext);
	if (!context) {
		throw new Error("Collapsible components must be used within a Collapsible");
	}
	return context;
}

interface CollapsibleProps {
	children: ReactNode;
	open?: boolean;
	defaultOpen?: boolean;
	onOpenChange?: (open: boolean) => void;
}

function Collapsible({
	children,
	open: controlledOpen,
	defaultOpen = false,
	onOpenChange: controlledOnOpenChange,
}: CollapsibleProps) {
	const [uncontrolledOpen, setUncontrolledOpen] = useState(defaultOpen);
	const open = controlledOpen ?? uncontrolledOpen;
	const onOpenChange = controlledOnOpenChange ?? setUncontrolledOpen;

	return (
		<CollapsibleContext.Provider value={{ open, onOpenChange }}>
			<div data-state={open ? "open" : "closed"}>{children}</div>
		</CollapsibleContext.Provider>
	);
}

interface CollapsibleTriggerProps {
	children: ReactNode;
	className?: string;
	asChild?: boolean;
}

function CollapsibleTrigger({ children, className }: CollapsibleTriggerProps) {
	const { open, onOpenChange } = useCollapsible();

	const handleClick = useCallback(() => {
		onOpenChange(!open);
	}, [open, onOpenChange]);

	return (
		<button
			type="button"
			onClick={handleClick}
			className={cn("w-full text-left", className)}
			data-state={open ? "open" : "closed"}
		>
			{children}
		</button>
	);
}

interface CollapsibleContentProps {
	children: ReactNode;
	className?: string;
}

function CollapsibleContent({ children, className }: CollapsibleContentProps) {
	const { open } = useCollapsible();

	if (!open) {
		return null;
	}

	return (
		<div
			className={cn(
				"overflow-hidden",
				"animate-in fade-in-0 slide-in-from-top-1 duration-200",
				className,
			)}
			data-state={open ? "open" : "closed"}
		>
			{children}
		</div>
	);
}

function CollapsibleArrow({ className }: { className?: string }) {
	const { open } = useCollapsible();

	return (
		<Icon
			name={open ? "chevron-down" : "chevron-right"}
			size="sm"
			className={cn(
				"text-muted-foreground transition-transform duration-200",
				className,
			)}
		/>
	);
}

Collapsible.Trigger = CollapsibleTrigger;
Collapsible.Content = CollapsibleContent;
Collapsible.Arrow = CollapsibleArrow;

export { Collapsible, useCollapsible };
