import type { RoadmapFeature } from "@/api/generated/model";
import { Badge } from "@/components/ui/badge";

type Props = {
	feature: RoadmapFeature;
	onClick: () => void;
	onDragStart: (e: React.DragEvent) => void;
};

const PRIORITY_COLORS: Record<string, string> = {
	must: "bg-red-500/10 text-red-500 border-red-500/30",
	should: "bg-orange-500/10 text-orange-500 border-orange-500/30",
	could: "bg-blue-500/10 text-blue-500 border-blue-500/30",
	wont: "bg-gray-500/10 text-gray-500 border-gray-500/30",
};

const COMPLEXITY_ICONS: Record<string, string> = {
	low: "1",
	medium: "2",
	high: "3",
};

export function FeatureCard({ feature, onClick, onDragStart }: Props) {
	const priority = feature.priority ?? "should";
	const complexity = feature.complexity ?? "medium";
	const priorityColor = PRIORITY_COLORS[priority] ?? PRIORITY_COLORS.should;

	return (
		<div
			draggable
			onClick={onClick}
			onDragStart={onDragStart}
			onKeyDown={(e) => {
				if (e.key === "Enter") {
					onClick();
				}
			}}
			role="button"
			tabIndex={0}
			className="bg-card rounded-lg border p-3 cursor-pointer hover:border-primary/50 transition-colors"
		>
			<div className="flex items-start gap-2 mb-2">
				<h4 className="font-medium text-sm flex-1 line-clamp-2">{feature.title}</h4>
				{feature.linked_task_id && (
					<svg
						className="w-4 h-4 text-green-500 shrink-0"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="2"
					>
						<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
						<path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
					</svg>
				)}
			</div>

			<p className="text-xs text-muted-foreground line-clamp-2 mb-3">{feature.description}</p>

			<div className="flex items-center justify-between">
				<Badge variant="outline" className={`text-[10px] ${priorityColor}`}>
					{priority.toUpperCase()}
				</Badge>

				<div className="flex items-center gap-2 text-xs text-muted-foreground">
					<span title={`Complexity: ${complexity}`}>
						{Array.from({ length: Number.parseInt(COMPLEXITY_ICONS[complexity] || "2", 10) })
							.map(() => "")
							.join("")}
					</span>
					{feature.impact === "high" && (
						<span title="High impact" className="text-yellow-500">
							<svg className="w-3 h-3" viewBox="0 0 24 24" fill="currentColor">
								<path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
							</svg>
						</span>
					)}
				</div>
			</div>
		</div>
	);
}
