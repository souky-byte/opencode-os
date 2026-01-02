import { cn } from "@/lib/utils";

interface LoaderProps {
	className?: string;
	size?: "sm" | "default" | "lg";
	message?: string;
}

function Loader({ className, size = "default", message }: LoaderProps) {
	const sizeClasses = {
		sm: "h-4 w-4 border-2",
		default: "h-8 w-8 border-2",
		lg: "h-12 w-12 border-3",
	};

	return (
		<div className={cn("flex flex-col items-center justify-center gap-3", className)}>
			<div
				className={cn(
					"animate-spin rounded-full border-muted-foreground border-t-primary",
					sizeClasses[size],
				)}
			/>
			{message && <p className="text-sm text-muted-foreground">{message}</p>}
		</div>
	);
}

export { Loader };
