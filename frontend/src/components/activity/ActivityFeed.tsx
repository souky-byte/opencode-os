import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { cn } from "@/lib/utils";
import type { SessionActivityMsg } from "@/types/generated/SessionActivityMsg";
import { ActivityItem } from "./ActivityItem";

interface ActivityFeedProps {
	activities: SessionActivityMsg[];
	isConnected: boolean;
	isFinished: boolean;
	error: string | null;
	className?: string;
}

export function ActivityFeed({
	activities,
	isConnected,
	isFinished,
	error,
	className,
}: ActivityFeedProps) {
	const scrollRef = useRef<HTMLDivElement>(null);
	const [autoScroll, setAutoScroll] = useState(true);
	const [isNearBottom, setIsNearBottom] = useState(true);

	const scrollToBottom = useCallback(() => {
		if (scrollRef.current) {
			scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
		}
	}, []);

	const handleScroll = useCallback(() => {
		if (!scrollRef.current) {
			return;
		}

		const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
		const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
		const nearBottom = distanceFromBottom < 100;

		setIsNearBottom(nearBottom);

		if (nearBottom && !autoScroll) {
			setAutoScroll(true);
		} else if (!nearBottom && autoScroll) {
			setAutoScroll(false);
		}
	}, [autoScroll]);

	useEffect(() => {
		if (autoScroll && activities.length > 0) {
			scrollToBottom();
		}
	}, [activities, autoScroll, scrollToBottom]);

	const handleResumeAutoScroll = () => {
		setAutoScroll(true);
		scrollToBottom();
	};

	const showLiveIndicator = isConnected && !isFinished;

	return (
		<div className={cn("flex flex-col h-full", className)}>
			<div className="flex items-center justify-between px-4 py-2 border-b border-border">
				<div className="flex items-center gap-2">
					<span className="text-sm font-medium">Activity</span>
					{showLiveIndicator && (
						<span className="flex items-center gap-1.5">
							<span className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
							<span className="text-xs text-green-500">Live</span>
						</span>
					)}
					{isFinished && <span className="text-xs text-muted-foreground">Completed</span>}
					{error && <span className="text-xs text-destructive">{error}</span>}
				</div>
				<span className="text-xs text-muted-foreground">{activities.length} events</span>
			</div>

			<ScrollArea className="flex-1" ref={scrollRef} onScrollCapture={handleScroll}>
				<div className="p-4 space-y-2">
					{activities.length === 0 ? (
						<div className="flex flex-col items-center justify-center py-12 text-center">
							{isConnected ? (
								<>
									<div className="w-8 h-8 border-2 border-muted-foreground/30 border-t-muted-foreground rounded-full animate-spin mb-3" />
									<p className="text-sm text-muted-foreground">Waiting for activity...</p>
								</>
							) : (
								<p className="text-sm text-muted-foreground">No activity recorded</p>
							)}
						</div>
					) : (
						activities.map((activity, index) => {
							const key = "id" in activity ? activity.id : `activity-${index}`;
							return <ActivityItem key={key} activity={activity} />;
						})
					)}
				</div>
			</ScrollArea>

			{!isNearBottom && (
				<div className="absolute bottom-4 left-1/2 -translate-x-1/2">
					<Button
						size="sm"
						variant="secondary"
						onClick={handleResumeAutoScroll}
						className="shadow-lg"
					>
						Scroll to bottom
					</Button>
				</div>
			)}
		</div>
	);
}
