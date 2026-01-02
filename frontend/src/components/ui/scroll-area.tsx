import type { HTMLAttributes } from "react";
import { forwardRef } from "react";
import { cn } from "@/lib/utils";

interface ScrollAreaProps extends HTMLAttributes<HTMLDivElement> {
	orientation?: "vertical" | "horizontal" | "both";
}

const ScrollArea = forwardRef<HTMLDivElement, ScrollAreaProps>(
	({ className, orientation = "vertical", children, ...props }, ref) => (
		<div
			ref={ref}
			className={cn(
				"relative",
				orientation === "vertical" && "overflow-y-auto overflow-x-hidden",
				orientation === "horizontal" && "overflow-x-auto overflow-y-hidden",
				orientation === "both" && "overflow-auto",
				className,
			)}
			{...props}
		>
			{children}
		</div>
	),
);
ScrollArea.displayName = "ScrollArea";

export { ScrollArea };
