import type { RoadmapFeature } from "@/api/generated/model";
import { useConvertFeatureToTask } from "@/api/generated/roadmap/roadmap";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Loader } from "@/components/ui/loader";

type Props = {
	feature: RoadmapFeature;
	onClose: () => void;
};

const PRIORITY_COLORS: Record<string, string> = {
	must: "bg-red-500/10 text-red-500 border-red-500/30",
	should: "bg-orange-500/10 text-orange-500 border-orange-500/30",
	could: "bg-blue-500/10 text-blue-500 border-blue-500/30",
	wont: "bg-gray-500/10 text-gray-500 border-gray-500/30",
};

export function FeatureDetailPanel({ feature, onClose }: Props) {
	const convertMutation = useConvertFeatureToTask();

	const handleConvertToTask = () => {
		convertMutation.mutate(
			{ featureId: feature.id },
			{
				onSuccess: () => {
					onClose();
				},
			},
		);
	};

	return (
		<>
			<div
				className="fixed inset-0 bg-background/80 backdrop-blur-sm z-40"
				onClick={onClose}
				onKeyDown={(e) => {
					if (e.key === "Escape") {
						onClose();
					}
				}}
				role="button"
				tabIndex={0}
				aria-label="Close panel"
			/>

			<div className="fixed right-0 top-0 bottom-0 w-full max-w-lg bg-card border-l z-50 overflow-y-auto">
				<div className="sticky top-0 bg-card/95 backdrop-blur-sm border-b p-4 flex items-center justify-between">
					<h2 className="font-semibold text-lg">Feature Details</h2>
					<button
						type="button"
						onClick={onClose}
						className="p-1 rounded-md hover:bg-accent transition-colors"
					>
						<svg
							className="w-5 h-5"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<path d="M18 6L6 18M6 6l12 12" />
						</svg>
					</button>
				</div>

				<div className="p-4 space-y-6">
					<div>
						<h3 className="text-xl font-bold mb-2">{feature.title}</h3>
						<p className="text-muted-foreground">{feature.description}</p>
					</div>

					<div className="flex flex-wrap gap-2">
						<Badge variant="outline" className={PRIORITY_COLORS[feature.priority ?? "should"]}>
							{(feature.priority ?? "should").toUpperCase()}
						</Badge>
						<Badge variant="outline">Complexity: {feature.complexity ?? "medium"}</Badge>
						<Badge variant="outline">Impact: {feature.impact ?? "medium"}</Badge>
					</div>

					{feature.rationale && (
						<div>
							<h4 className="font-semibold mb-2">Rationale</h4>
							<p className="text-sm text-muted-foreground">{feature.rationale}</p>
						</div>
					)}

					{feature.acceptance_criteria && feature.acceptance_criteria.length > 0 && (
						<div>
							<h4 className="font-semibold mb-2">Acceptance Criteria</h4>
							<ul className="list-disc list-inside space-y-1 text-sm text-muted-foreground">
								{feature.acceptance_criteria.map((criteria, i) => (
									<li key={i}>{criteria}</li>
								))}
							</ul>
						</div>
					)}

					{feature.user_stories && feature.user_stories.length > 0 && (
						<div>
							<h4 className="font-semibold mb-2">User Stories</h4>
							<ul className="space-y-2 text-sm">
								{feature.user_stories.map((story, i) => (
									<li key={i} className="p-2 bg-muted rounded-md italic">
										{story}
									</li>
								))}
							</ul>
						</div>
					)}

					{feature.dependencies && feature.dependencies.length > 0 && (
						<div>
							<h4 className="font-semibold mb-2">Dependencies</h4>
							<div className="flex flex-wrap gap-1">
								{feature.dependencies.map((dep) => (
									<Badge key={dep} variant="secondary" className="text-xs">
										{dep}
									</Badge>
								))}
							</div>
						</div>
					)}

					<div className="border-t pt-4">
						{feature.linked_task_id ? (
							<div className="flex items-center gap-2 text-sm text-green-500">
								<svg
									className="w-4 h-4"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									strokeWidth="2"
								>
									<path d="M9 12l2 2 4-4" />
									<circle cx="12" cy="12" r="10" />
								</svg>
								Linked to task: {feature.linked_task_id}
							</div>
						) : (
							<Button
								onClick={handleConvertToTask}
								disabled={convertMutation.isPending}
								className="w-full"
							>
								{convertMutation.isPending ? (
									<>
										<Loader className="w-4 h-4 mr-2" />
										Converting...
									</>
								) : (
									<>
										<svg
											className="w-4 h-4 mr-2"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											strokeWidth="2"
										>
											<path d="M12 5v14M5 12h14" />
										</svg>
										Convert to Task
									</>
								)}
							</Button>
						)}
					</div>
				</div>
			</div>
		</>
	);
}
