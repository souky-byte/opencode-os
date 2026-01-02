import { type ReactNode, useState, useEffect } from "react";
import { cn } from "@/lib/utils";
import { Collapsible } from "@/components/ui/collapsible";
import { Icon, type IconName } from "@/components/ui/icon";

export interface TriggerTitle {
	title: string;
	subtitle?: string;
	args?: string[];
	action?: ReactNode;
}

interface BasicToolProps {
	icon: IconName;
	trigger: TriggerTitle | ReactNode;
	children?: ReactNode;
	status?: "pending" | "running" | "completed" | "error";
	hideDetails?: boolean;
	defaultOpen?: boolean;
	forceOpen?: boolean;
	className?: string;
}

function isTriggerTitle(val: unknown): val is TriggerTitle {
	return (
		typeof val === "object" &&
		val !== null &&
		"title" in val &&
		typeof (val as TriggerTitle).title === "string"
	);
}

export function BasicTool({
	icon,
	trigger,
	children,
	status = "completed",
	hideDetails,
	defaultOpen = false,
	forceOpen,
	className,
}: BasicToolProps) {
	const [open, setOpen] = useState(defaultOpen);

	useEffect(() => {
		if (forceOpen) {
			setOpen(true);
		}
	}, [forceOpen]);

	const isRunning = status === "running";
	const hasError = status === "error";

	return (
		<Collapsible open={open} onOpenChange={setOpen}>
			<Collapsible.Trigger>
				<div
					className={cn(
						"group flex items-center gap-2 rounded-lg px-3 py-2",
						"bg-muted/30 hover:bg-muted/50 transition-colors",
						"border border-transparent",
						hasError && "border-red-500/30 bg-red-500/5",
						isRunning && "border-primary/30 bg-primary/5",
						className,
					)}
				>
					<div className="flex items-center gap-2 flex-1 min-w-0">
						<div
							className={cn(
								"shrink-0 p-1 rounded",
								hasError && "text-red-400",
								isRunning && "text-primary",
								!hasError && !isRunning && "text-muted-foreground",
							)}
						>
							{isRunning ? (
								<Icon name="loading" size="sm" spin />
							) : (
								<Icon name={icon} size="sm" />
							)}
						</div>

						<div className="flex-1 min-w-0">
							{isTriggerTitle(trigger) ? (
								<div className="flex items-center gap-2 min-w-0">
									<span className="text-sm font-medium text-foreground shrink-0">
										{trigger.title}
									</span>
									{trigger.subtitle && (
										<span className="text-sm text-muted-foreground truncate">
											{trigger.subtitle}
										</span>
									)}
									{trigger.args && trigger.args.length > 0 && (
										<div className="flex gap-1 shrink-0">
											{trigger.args.map((arg, i) => (
												<span
													key={`${arg}-${i}`}
													className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground font-mono"
												>
													{arg}
												</span>
											))}
										</div>
									)}
									{trigger.action && (
										<div className="ml-auto shrink-0">{trigger.action}</div>
									)}
								</div>
							) : (
								trigger
							)}
						</div>
					</div>

					{children && !hideDetails && (
						<Collapsible.Arrow className="shrink-0 opacity-50 group-hover:opacity-100" />
					)}
				</div>
			</Collapsible.Trigger>

			{children && !hideDetails && (
				<Collapsible.Content className="mt-1 ml-6 pl-3 border-l border-border/50">
					{children}
				</Collapsible.Content>
			)}
		</Collapsible>
	);
}

export function GenericTool({
	tool,
	status,
}: { tool: string; status?: "pending" | "running" | "completed" | "error" }) {
	return (
		<BasicTool
			icon="tool"
			trigger={{ title: tool }}
			status={status}
			hideDetails
		/>
	);
}
