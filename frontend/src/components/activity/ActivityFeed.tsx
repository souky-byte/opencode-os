import { useCallback, useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import type { SessionActivityMsg } from "@/types/generated/SessionActivityMsg";
import { ActivityItem } from "./ActivityItem";
import { Icon } from "@/components/ui/icon";

interface ActivityFeedProps {
  activities: SessionActivityMsg[];
  isConnected: boolean;
  className?: string;
}

export function ActivityFeed({
  activities,
  isConnected,
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
    if (!scrollRef.current) return;

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

  return (
    <div className={cn("flex flex-col h-full relative", className)}>
      {/* Activity list */}
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto overflow-x-hidden"
      >
        <div className="px-4 py-2 space-y-0.5">
          {activities.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              {isConnected ? (
                <>
                  <Icon
                    name="loading"
                    size="sm"
                    spin
                    className="text-muted-foreground/40 mb-2"
                  />
                  <p className="text-xs text-muted-foreground/50">
                    Waiting for activity…
                  </p>
                </>
              ) : (
                <p className="text-xs text-muted-foreground/50">No activity</p>
              )}
            </div>
          ) : (
            activities.map((activity, index) => {
              const key = "id" in activity ? activity.id : `activity-${index}`;
              return <ActivityItem key={key} activity={activity} />;
            })
          )}
        </div>
      </div>

      {/* Scroll to bottom button */}
      {!isNearBottom && (
        <button
          type="button"
          onClick={handleResumeAutoScroll}
          className="absolute bottom-3 left-1/2 -translate-x-1/2 flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-muted-foreground bg-card border border-border rounded-full shadow-lg hover:bg-accent transition-colors"
        >
          <Icon name="chevron-down" size="xs" />
          <span>Scroll to bottom</span>
        </button>
      )}
    </div>
  );
}
