import { useMemo, useState } from "react";
import type { Roadmap, RoadmapFeature, RoadmapFeatureStatus } from "@/api/generated/model";
import { useUpdateFeature } from "@/api/generated/roadmap/roadmap";
import { Button } from "@/components/ui/button";
import { FeatureCard } from "./FeatureCard";
import { FeatureDetailPanel } from "./FeatureDetailPanel";

type Props = {
	roadmap: Roadmap;
	onRegenerate: () => void;
};

const COLUMNS: { status: RoadmapFeatureStatus; label: string; color: string }[] = [
	{ status: "under_review", label: "Under Review", color: "bg-yellow-500/10 border-yellow-500/30" },
	{ status: "planned", label: "Planned", color: "bg-blue-500/10 border-blue-500/30" },
	{ status: "in_progress", label: "In Progress", color: "bg-purple-500/10 border-purple-500/30" },
	{ status: "done", label: "Done", color: "bg-green-500/10 border-green-500/30" },
];

export function RoadmapKanban({ roadmap, onRegenerate }: Props) {
	const [selectedFeature, setSelectedFeature] = useState<RoadmapFeature | null>(null);
	const updateFeature = useUpdateFeature();

	const featuresByStatus = useMemo(() => {
		const grouped: Record<RoadmapFeatureStatus, RoadmapFeature[]> = {
			under_review: [],
			planned: [],
			in_progress: [],
			done: [],
		};

		const features = roadmap.features ?? [];
		for (const feature of features) {
			const status = feature.status ?? "under_review";
			if (grouped[status]) {
				grouped[status].push(feature);
			}
		}

		return grouped;
	}, [roadmap.features]);

	const handleDragStart = (e: React.DragEvent, feature: RoadmapFeature) => {
		e.dataTransfer.setData("featureId", feature.id);
		e.dataTransfer.effectAllowed = "move";
	};

	const handleDragOver = (e: React.DragEvent) => {
		e.preventDefault();
		e.dataTransfer.dropEffect = "move";
	};

	const handleDrop = (e: React.DragEvent, targetStatus: RoadmapFeatureStatus) => {
		e.preventDefault();
		const featureId = e.dataTransfer.getData("featureId");

		const features = roadmap.features ?? [];
		const feature = features.find((f) => f.id === featureId);
		if (feature && feature.status !== targetStatus) {
			updateFeature.mutate({
				featureId,
				data: { status: targetStatus },
			});
		}
	};

	return (
		<div className="flex flex-col h-full">
			<div className="flex items-center justify-between p-4 border-b">
				<div>
					<h1 className="text-xl font-bold">{roadmap.project_name} Roadmap</h1>
					<p className="text-sm text-muted-foreground">{roadmap.vision}</p>
				</div>
				<Button variant="outline" size="sm" onClick={onRegenerate}>
					Regenerate
				</Button>
			</div>

			<div className="flex-1 overflow-x-auto p-4">
				<div className="flex gap-4 min-w-fit h-full">
					{COLUMNS.map(({ status, label, color }) => (
						<div
							key={status}
							className={`flex flex-col w-80 shrink-0 rounded-lg border ${color}`}
							onDragOver={handleDragOver}
							onDrop={(e) => handleDrop(e, status)}
						>
							<div className="flex items-center justify-between p-3 border-b border-inherit">
								<h3 className="font-semibold text-sm">{label}</h3>
								<span className="text-xs text-muted-foreground bg-background px-2 py-0.5 rounded-full">
									{featuresByStatus[status].length}
								</span>
							</div>

							<div className="flex-1 overflow-y-auto p-2 space-y-2">
								{featuresByStatus[status].map((feature) => (
									<FeatureCard
										key={feature.id}
										feature={feature}
										onClick={() => setSelectedFeature(feature)}
										onDragStart={(e) => handleDragStart(e, feature)}
									/>
								))}

								{featuresByStatus[status].length === 0 && (
									<div className="flex items-center justify-center h-24 text-sm text-muted-foreground">
										No features
									</div>
								)}
							</div>
						</div>
					))}
				</div>
			</div>

			{selectedFeature && (
				<FeatureDetailPanel feature={selectedFeature} onClose={() => setSelectedFeature(null)} />
			)}
		</div>
	);
}
