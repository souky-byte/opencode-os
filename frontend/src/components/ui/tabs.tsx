import type { ReactNode } from "react";
import { createContext, useContext, useState } from "react";
import { cn } from "@/lib/utils";

interface TabsContextValue {
	value: string;
	onValueChange: (value: string) => void;
}

const TabsContext = createContext<TabsContextValue | null>(null);

function useTabsContext() {
	const context = useContext(TabsContext);
	if (!context) {
		throw new Error("Tabs components must be used within a Tabs");
	}
	return context;
}

interface TabsProps {
	children: ReactNode;
	defaultValue?: string;
	value?: string;
	onValueChange?: (value: string) => void;
	className?: string;
}

function Tabs({
	children,
	defaultValue = "",
	value: controlledValue,
	onValueChange: controlledOnValueChange,
	className,
}: TabsProps) {
	const [uncontrolledValue, setUncontrolledValue] = useState(defaultValue);
	const value = controlledValue ?? uncontrolledValue;
	const onValueChange = controlledOnValueChange ?? setUncontrolledValue;

	return (
		<TabsContext.Provider value={{ value, onValueChange }}>
			<div className={className}>{children}</div>
		</TabsContext.Provider>
	);
}

interface TabsListProps {
	children: ReactNode;
	className?: string;
}

function TabsList({ children, className }: TabsListProps) {
	return (
		<div
			className={cn(
				"inline-flex h-10 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground",
				className,
			)}
		>
			{children}
		</div>
	);
}

interface TabsTriggerProps {
	children: ReactNode;
	value: string;
	className?: string;
	disabled?: boolean;
}

function TabsTrigger({ children, value, className, disabled }: TabsTriggerProps) {
	const { value: selectedValue, onValueChange } = useTabsContext();
	const isSelected = selectedValue === value;

	return (
		<button
			type="button"
			className={cn(
				"inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1.5 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
				isSelected && "bg-background text-foreground shadow-sm",
				className,
			)}
			disabled={disabled}
			onClick={() => onValueChange(value)}
		>
			{children}
		</button>
	);
}

interface TabsContentProps {
	children: ReactNode;
	value: string;
	className?: string;
}

function TabsContent({ children, value, className }: TabsContentProps) {
	const { value: selectedValue } = useTabsContext();

	if (selectedValue !== value) {
		return null;
	}

	return (
		<div
			className={cn(
				"mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
				className,
			)}
		>
			{children}
		</div>
	);
}

export { Tabs, TabsList, TabsTrigger, TabsContent };
