import type { InputHTMLAttributes } from "react";
import { forwardRef } from "react";
import { cn } from "@/lib/utils";

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {}

const Input = forwardRef<HTMLInputElement, InputProps>(({ className, type, ...props }, ref) => (
	<input
		type={type}
		className={cn(
			"flex h-10 w-full rounded-lg border border-border bg-muted/50 px-3 py-2",
			"text-sm text-foreground",
			"ring-offset-background transition-colors",
			"file:border-0 file:bg-transparent file:text-sm file:font-medium",
			"placeholder:text-muted-foreground",
			"hover:border-border/80 hover:bg-muted/70",
			"focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:border-primary/50",
			"disabled:cursor-not-allowed disabled:opacity-50",
			className,
		)}
		ref={ref}
		{...props}
	/>
));
Input.displayName = "Input";

export { Input };
