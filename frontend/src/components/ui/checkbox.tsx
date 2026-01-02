import * as React from "react";
import { cn } from "@/lib/utils";
import { Icon } from "./icon";

interface CheckboxProps extends Omit<
  React.ButtonHTMLAttributes<HTMLButtonElement>,
  "onChange"
> {
  checked?: boolean;
  indeterminate?: boolean;
  onCheckedChange?: (checked: boolean) => void;
}

const Checkbox = React.forwardRef<HTMLButtonElement, CheckboxProps>(
  ({ className, checked, indeterminate, onCheckedChange, ...props }, ref) => {
    return (
      <button
        ref={ref}
        type="button"
        role="checkbox"
        aria-checked={indeterminate ? "mixed" : checked}
        onClick={() => onCheckedChange?.(!checked)}
        className={cn(
          "peer h-4 w-4 shrink-0 rounded-sm border border-primary flex items-center justify-center",
          "ring-offset-background focus-visible:outline-none focus-visible:ring-2",
          "focus-visible:ring-ring focus-visible:ring-offset-2",
          "disabled:cursor-not-allowed disabled:opacity-50",
          (checked || indeterminate) && "bg-primary text-primary-foreground",
          className,
        )}
        {...props}
      >
        {checked && !indeterminate && (
          <Icon name="check" size="sm" className="text-current" />
        )}
        {indeterminate && (
          <Icon name="minus" size="sm" className="text-current" />
        )}
      </button>
    );
  },
);
Checkbox.displayName = "Checkbox";

export { Checkbox };
