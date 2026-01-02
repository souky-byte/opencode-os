import type { HTMLAttributes } from "react";
import { cn } from "@/lib/utils";

export interface BadgeProps extends HTMLAttributes<HTMLDivElement> {
	variant?: "default" | "secondary" | "destructive" | "outline" | "success" | "warning";
}

function Badge({ className, variant = "default", ...props }: BadgeProps) {
	return (
		<div
			className={cn(
				"inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
				variant === "default" &&
					"border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
				variant === "secondary" &&
					"border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
				variant === "destructive" &&
					"border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
				variant === "outline" && "text-foreground",
				variant === "success" &&
					"border-transparent bg-success text-success-foreground hover:bg-success/80",
				variant === "warning" &&
					"border-transparent bg-warning text-warning-foreground hover:bg-warning/80",
				className,
			)}
			{...props}
		/>
	);
}

export { Badge };
