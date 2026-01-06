import { useEffect, useRef, useState } from "react";
import { useGetGenerationStatus } from "@/api/generated/roadmap/roadmap";
import { Loader } from "@/components/ui/loader";

const PHASE_LABELS: Record<string, string> = {
	idle: "Preparing...",
	analyzing: "Analyzing project structure...",
	discovering: "Discovering target audience & product vision...",
	generating: "Generating feature roadmap...",
	complete: "Complete!",
	error: "Generation failed",
};

interface GenerationStatus {
	phase: string;
	progress: number;
	message: string;
}

export function RoadmapGenerationProgress() {
	const [dots, setDots] = useState("");
	const [status, setStatus] = useState<GenerationStatus>({
		phase: "analyzing",
		progress: 10,
		message: "Analyzing project structure...",
	});
	const eventSourceRef = useRef<EventSource | null>(null);

	// Fetch initial status on mount (one-time, not polling)
	const { data: initialStatus } = useGetGenerationStatus({
		query: {
			staleTime: 0,
			refetchOnWindowFocus: false,
		},
	});

	// Update status when initial data arrives
	useEffect(() => {
		if (initialStatus?.data) {
			const data = initialStatus.data;
			setStatus({
				phase: data.phase ?? "analyzing",
				progress: data.progress ?? 10,
				message: data.message ?? "Working...",
			});
		}
	}, [initialStatus]);

	// Animate dots
	useEffect(() => {
		const interval = setInterval(() => {
			setDots((d) => (d.length >= 3 ? "" : `${d}.`));
		}, 500);
		return () => clearInterval(interval);
	}, []);

	// Listen to SSE events for progress updates
	useEffect(() => {
		const baseUrl = import.meta.env.VITE_API_URL || "";
		const url = `${baseUrl}/api/events`;
		const eventSource = new EventSource(url);
		eventSourceRef.current = eventSource;

		const handleProgressEvent = (e: MessageEvent) => {
			try {
				const data = JSON.parse(e.data);
				const event = data?.event;

				if (event?.type === "roadmap.generation_progress") {
					setStatus({
						phase: event.phase ?? "idle",
						progress: event.progress ?? 0,
						message: event.message ?? "Working...",
					});
				}
			} catch {
				// Silently ignore JSON parse errors from SSE
			}
		};

		eventSource.addEventListener("roadmap.generation_progress", handleProgressEvent);

		return () => {
			eventSource.close();
			eventSourceRef.current = null;
		};
	}, []);

	const phase = status.phase;
	const progress = status.progress;
	const message = PHASE_LABELS[phase] ?? status.message ?? "Working...";

	return (
		<div className="flex h-full flex-col items-center justify-center p-8">
			<div className="w-full max-w-md">
				<div className="flex flex-col items-center mb-8">
					<div className="relative">
						<Loader className="h-16 w-16 text-primary" />
					</div>
				</div>

				<h2 className="text-xl font-bold text-center mb-2">Generating Roadmap{dots}</h2>
				<p className="text-muted-foreground text-center mb-6">{message}</p>

				<div className="w-full bg-muted rounded-full h-2 overflow-hidden">
					<div
						className="h-full bg-primary transition-all duration-500 ease-out"
						style={{ width: `${progress}%` }}
					/>
				</div>

				<div className="flex justify-between mt-2 text-xs text-muted-foreground">
					<span>Progress</span>
					<span>{progress}%</span>
				</div>

				<div className="mt-8 space-y-2">
					{["analyzing", "discovering", "generating"].map((p) => {
						const isActive = p === phase;
						const isDone =
							(p === "analyzing" && ["discovering", "generating", "complete"].includes(phase)) ||
							(p === "discovering" && ["generating", "complete"].includes(phase)) ||
							(p === "generating" && phase === "complete");

						return (
							<div
								key={p}
								className={`flex items-center gap-3 px-3 py-2 rounded-md transition-colors ${
									isActive
										? "bg-primary/10 text-primary"
										: isDone
											? "text-muted-foreground"
											: "text-muted-foreground/50"
								}`}
							>
								{isDone ? (
									<svg className="w-4 h-4 text-green-500" viewBox="0 0 24 24" fill="currentColor">
										<path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z" />
									</svg>
								) : isActive ? (
									<div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
								) : (
									<div className="w-4 h-4 border-2 border-muted-foreground/30 rounded-full" />
								)}
								<span className="text-sm capitalize">{p.replace("_", " ")}</span>
							</div>
						);
					})}
				</div>
			</div>
		</div>
	);
}
