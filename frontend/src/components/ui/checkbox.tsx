import { forwardRef, useEffect, useRef } from "react";
import { cn } from "@/lib/utils";
import { Icon } from "./icon";

export interface CheckboxProps {
	checked?: boolean;
	indeterminate?: boolean;
	onCheckedChange?: (checked: boolean) => void;
	disabled?: boolean;
	className?: string;
	id?: string;
}

const Checkbox = forwardRef<HTMLButtonElement, CheckboxProps>(
	({ checked = false, indeterminate = false, onCheckedChange, disabled, className, id }, ref) => {
		const internalRef = useRef<HTMLButtonElement>(null);
		const resolvedRef = (ref as React.RefObject<HTMLButtonElement>) || internalRef;

		useEffect(() => {
			if (resolvedRef.current) {
				(resolvedRef.current as HTMLButtonElement & { indeterminate?: boolean }).indeterminate = indeterminate;
			}
		}, [indeterminate, resolvedRef]);

		return (
			<button
				ref={resolvedRef}
				type="button"
				role="checkbox"
				id={id}
				aria-checked={indeterminate ? "mixed" : checked}
				disabled={disabled}
				onClick={() => onCheckedChange?.(!checked)}
				className={cn(
					"peer h-4 w-4 shrink-0 rounded border border-border",
					"ring-offset-background transition-colors",
					"focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
					"disabled:cursor-not-allowed disabled:opacity-50",
					(checked || indeterminate) && "bg-primary border-primary text-primary-foreground",
					className,
				)}
			>
				{(checked || indeterminate) && (
					<span className="flex items-center justify-center text-current">
						<Icon
							name={indeterminate ? "minus" : "check"}
							size="xs"
							className="text-primary-foreground"
						/>
					</span>
				)}
			</button>
		);
	},
);
Checkbox.displayName = "Checkbox";

export { Checkbox };
