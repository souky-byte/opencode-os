import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import {
	getGetGenerationStatusQueryKey,
	getGetRoadmapQueryKey,
	useGenerateRoadmap,
	useGetGenerationStatus,
	useGetRoadmap,
} from "@/api/generated/roadmap/roadmap";
import { Loader } from "@/components/ui/loader";
import { RoadmapEmptyState } from "./RoadmapEmptyState";
import { RoadmapGenerationProgress } from "./RoadmapGenerationProgress";
import { RoadmapKanban } from "./RoadmapKanban";

export function RoadmapView() {
	const queryClient = useQueryClient();
	const [isGenerating, setIsGenerating] = useState(false);
	const justStartedRef = useRef(false);
	const eventSourceRef = useRef<EventSource | null>(null);

	const { data: roadmapData, isLoading, refetch } = useGetRoadmap();
	const { data: statusData } = useGetGenerationStatus({
		query: {
			staleTime: 0,
			refetchOnWindowFocus: false,
		},
	});
	const generateMutation = useGenerateRoadmap();

	// Check initial status - if server says generation is active, show progress UI
	useEffect(() => {
		if (statusData?.data) {
			const phase = statusData.data.phase;
			const isActive = phase === "analyzing" || phase === "discovering" || phase === "generating";

			if (isActive) {
				// Server shows active generation, update our state
				setIsGenerating(true);
				justStartedRef.current = false;
			} else if (phase === "idle" && isGenerating && !justStartedRef.current) {
				// Server reset to idle, but only if we didn't just start
				// (avoids race condition where stale status resets our state)
				setIsGenerating(false);
			}
		}
	}, [statusData, isGenerating]);

	useEffect(() => {
		const baseUrl = import.meta.env.VITE_API_URL || "";
		const url = `${baseUrl}/api/events`;
		const eventSource = new EventSource(url);
		eventSourceRef.current = eventSource;

		const handleRoadmapEvent = (e: MessageEvent) => {
			try {
				const data = JSON.parse(e.data);
				const eventType = data?.event?.type;

				if (eventType === "roadmap.generation_started") {
					justStartedRef.current = false;
					setIsGenerating(true);
					// Invalidate status query to get fresh data
					void queryClient.invalidateQueries({ queryKey: getGetGenerationStatusQueryKey() });
				} else if (eventType === "roadmap.generation_progress") {
					justStartedRef.current = false;
					setIsGenerating(true);
					// Invalidate status query to get fresh data
					void queryClient.invalidateQueries({ queryKey: getGetGenerationStatusQueryKey() });
				} else if (
					eventType === "roadmap.generation_completed" ||
					eventType === "roadmap.generation_failed"
				) {
					justStartedRef.current = false;
					setIsGenerating(false);
					void queryClient.invalidateQueries({ queryKey: getGetGenerationStatusQueryKey() });
					void refetch();
				} else if (
					eventType === "roadmap.feature_updated" ||
					eventType === "roadmap.feature_converted"
				) {
					void queryClient.invalidateQueries({ queryKey: getGetRoadmapQueryKey() });
				}
			} catch {
				// Silently ignore JSON parse errors from SSE
			}
		};

		eventSource.addEventListener("roadmap.generation_started", handleRoadmapEvent);
		eventSource.addEventListener("roadmap.generation_progress", handleRoadmapEvent);
		eventSource.addEventListener("roadmap.generation_completed", handleRoadmapEvent);
		eventSource.addEventListener("roadmap.generation_failed", handleRoadmapEvent);
		eventSource.addEventListener("roadmap.feature_updated", handleRoadmapEvent);
		eventSource.addEventListener("roadmap.feature_converted", handleRoadmapEvent);

		return () => {
			eventSource.close();
			eventSourceRef.current = null;
		};
	}, [queryClient, refetch]);

	const handleGenerate = () => {
		justStartedRef.current = true;
		setIsGenerating(true);
		generateMutation.mutate(
			{ data: { force: true } },
			{
				onError: () => {
					justStartedRef.current = false;
					setIsGenerating(false);
				},
			},
		);
	};

	const handleRegenerate = () => {
		justStartedRef.current = true;
		setIsGenerating(true);
		generateMutation.mutate(
			{ data: { force: true } },
			{
				onError: () => {
					justStartedRef.current = false;
					setIsGenerating(false);
				},
			},
		);
	};

	if (isLoading) {
		return (
			<div className="flex h-full items-center justify-center">
				<Loader />
			</div>
		);
	}

	if (isGenerating) {
		return <RoadmapGenerationProgress />;
	}

	const roadmap = roadmapData?.data?.roadmap;

	if (!roadmap) {
		return <RoadmapEmptyState onGenerate={handleGenerate} isLoading={generateMutation.isPending} />;
	}

	return <RoadmapKanban roadmap={roadmap} onRegenerate={handleRegenerate} />;
}
