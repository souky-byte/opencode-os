import { Button } from "@/components/ui/button";
import { Loader } from "@/components/ui/loader";

type Props = {
	onGenerate: () => void;
	isLoading: boolean;
};

export function RoadmapEmptyState({ onGenerate, isLoading }: Props) {
	return (
		<div className="flex h-full flex-col items-center justify-center p-8 text-center">
			<div className="rounded-full bg-primary/10 p-6 mb-6">
				<svg
					className="w-12 h-12 text-primary"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					strokeWidth="1.5"
				>
					<path d="M9 11l3 3L22 4" />
					<path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11" />
				</svg>
			</div>

			<h2 className="text-2xl font-bold mb-2">No Roadmap Yet</h2>
			<p className="text-muted-foreground max-w-md mb-8">
				Generate an AI-powered roadmap to discover features, prioritize work, and plan your
				project&apos;s future. The roadmap will analyze your codebase and suggest features based on
				your target audience.
			</p>

			<Button onClick={onGenerate} disabled={isLoading} size="lg" className="gap-2">
				{isLoading ? (
					<>
						<Loader className="h-4 w-4" />
						Generating...
					</>
				) : (
					<>
						<svg
							className="w-5 h-5"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<path d="M12 5v14M5 12h14" />
						</svg>
						Generate Roadmap
					</>
				)}
			</Button>
		</div>
	);
}
