import type { ButtonHTMLAttributes } from "react";
import { forwardRef } from "react";
import { cn } from "@/lib/utils";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
	variant?: "default" | "destructive" | "outline" | "secondary" | "ghost" | "link" | "success";
	size?: "default" | "sm" | "lg" | "icon";
}

const Button = forwardRef<HTMLButtonElement, ButtonProps>(
	({ className, variant = "default", size = "default", ...props }, ref) => (
		<button
			className={cn(
				// Base styles
				"inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium",
				"ring-offset-background transition-all duration-150",
				"focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
				"disabled:pointer-events-none disabled:opacity-50",
				// Variants
				variant === "default" && [
					"bg-primary text-primary-foreground",
					"hover:bg-primary/90 hover:shadow-lg hover:shadow-primary/20",
					"active:scale-[0.98]",
				],
				variant === "destructive" && [
					"bg-destructive text-destructive-foreground",
					"hover:bg-destructive/90 hover:shadow-lg hover:shadow-destructive/20",
					"active:scale-[0.98]",
				],
				variant === "success" && [
					"bg-green-600 text-white",
					"hover:bg-green-500 hover:shadow-lg hover:shadow-green-500/20",
					"active:scale-[0.98]",
				],
				variant === "outline" && [
					"border border-border bg-transparent text-foreground",
					"hover:bg-accent hover:border-border/80",
				],
				variant === "secondary" && [
					"bg-secondary text-secondary-foreground",
					"hover:bg-secondary/80",
				],
				variant === "ghost" && [
					"text-muted-foreground",
					"hover:bg-accent hover:text-foreground",
				],
				variant === "link" && [
					"text-primary underline-offset-4",
					"hover:underline",
				],
				// Sizes
				size === "default" && "h-10 px-4 py-2",
				size === "sm" && "h-8 px-3 text-xs",
				size === "lg" && "h-12 px-6 text-base",
				size === "icon" && "h-10 w-10",
				className,
			)}
			ref={ref}
			{...props}
		/>
	),
);
Button.displayName = "Button";

export { Button };
