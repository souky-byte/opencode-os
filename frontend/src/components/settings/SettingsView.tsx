import { useState } from "react";
import { cn } from "@/lib/utils";
import { GitHubSettings } from "./GitHubSettings";
import { ModelSettings } from "./ModelSettings";
import { RoadmapSettings } from "./RoadmapSettings";
import { WikiSettings } from "@/components/wiki/WikiSettings";

type SettingsCategory = "integrations" | "models" | "wiki" | "roadmap";

const CATEGORIES: { id: SettingsCategory; label: string; icon: React.ReactNode }[] = [
	{
		id: "integrations",
		label: "Integrations",
		icon: (
			<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
				<path d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101" />
				<path d="M10.172 13.828a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
			</svg>
		),
	},
	{
		id: "models",
		label: "Development Models",
		icon: (
			<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
				<path d="M12 2L2 7l10 5 10-5-10-5z" />
				<path d="M2 17l10 5 10-5" />
				<path d="M2 12l10 5 10-5" />
			</svg>
		),
	},
	{
		id: "wiki",
		label: "Wiki",
		icon: (
			<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
				<path d="M4 19.5A2.5 2.5 0 016.5 17H20" />
				<path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z" />
				<path d="M8 7h8" />
				<path d="M8 11h8" />
				<path d="M8 15h5" />
			</svg>
		),
	},
	{
		id: "roadmap",
		label: "Roadmap",
		icon: (
			<svg className="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5">
				<path d="M9 11l3 3L22 4" />
				<path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11" />
			</svg>
		),
	},
];

const CATEGORY_TITLES: Record<SettingsCategory, { title: string; description: string }> = {
	integrations: {
		title: "Integrations",
		description: "Connect external services and configure API tokens.",
	},
	models: {
		title: "Development Models",
		description: "Configure which AI models to use for each development phase.",
	},
	wiki: {
		title: "Wiki",
		description: "Configure AI-powered documentation generation and code search.",
	},
	roadmap: {
		title: "Roadmap",
		description: "Configure AI model for roadmap generation.",
	},
};

export function SettingsView() {
	const [activeCategory, setActiveCategory] = useState<SettingsCategory>("integrations");

	const { title, description } = CATEGORY_TITLES[activeCategory];

	return (
		<div className="flex h-full">
			{/* Left sidebar */}
			<div className="w-56 shrink-0 border-r border-border bg-card/30">
				<div className="p-4 border-b border-border">
					<h1 className="text-lg font-semibold">Settings</h1>
				</div>
				<nav className="p-2 space-y-0.5">
					{CATEGORIES.map((category) => (
						<button
							key={category.id}
							type="button"
							onClick={() => setActiveCategory(category.id)}
							className={cn(
								"flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm transition-colors",
								activeCategory === category.id
									? "bg-primary/10 text-primary font-medium"
									: "text-muted-foreground hover:bg-accent hover:text-foreground",
							)}
						>
							<span className={activeCategory === category.id ? "text-primary" : ""}>
								{category.icon}
							</span>
							{category.label}
						</button>
					))}
				</nav>
			</div>

			{/* Main content */}
			<div className="flex-1 overflow-auto">
				<div className="p-6">
					<div className="max-w-3xl">
						<div className="mb-6">
							<h2 className="text-xl font-semibold">{title}</h2>
							<p className="mt-1 text-sm text-muted-foreground">{description}</p>
						</div>

						<div className="space-y-6">
							{activeCategory === "integrations" && <GitHubSettings />}
							{activeCategory === "models" && <ModelSettings />}
							{activeCategory === "wiki" && <WikiSettings />}
							{activeCategory === "roadmap" && <RoadmapSettings />}
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
