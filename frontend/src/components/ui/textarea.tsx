import type { TextareaHTMLAttributes } from "react";
import { forwardRef } from "react";
import { cn } from "@/lib/utils";

export interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {}

const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(({ className, ...props }, ref) => (
	<textarea
		className={cn(
			"flex min-h-[80px] w-full rounded-lg border border-border bg-muted/50 px-3 py-2",
			"text-sm text-foreground resize-none",
			"ring-offset-background transition-colors",
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
Textarea.displayName = "Textarea";

export { Textarea };
