import { useEffect, useRef, useState } from "react";

interface MermaidDiagramProps {
	chart: string;
}

export function MermaidDiagram({ chart }: MermaidDiagramProps) {
	const containerRef = useRef<HTMLDivElement>(null);
	const [svg, setSvg] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [isLoading, setIsLoading] = useState(true);

	useEffect(() => {
		let isMounted = true;

		const renderDiagram = async () => {
			try {
				setIsLoading(true);
				setError(null);

				// Dynamically import mermaid to avoid SSR issues
				const mermaid = await import("mermaid");

				// Initialize mermaid with dark theme
				mermaid.default.initialize({
					startOnLoad: false,
					theme: "dark",
					securityLevel: "loose",
					fontFamily: "ui-monospace, monospace",
					flowchart: {
						curve: "basis",
						padding: 20,
					},
					themeVariables: {
						primaryColor: "#3b82f6",
						primaryTextColor: "#f8fafc",
						primaryBorderColor: "#3b82f6",
						lineColor: "#64748b",
						secondaryColor: "#1e293b",
						tertiaryColor: "#0f172a",
						background: "#0f172a",
						mainBkg: "#1e293b",
						nodeBorder: "#3b82f6",
						clusterBkg: "#1e293b",
						titleColor: "#f8fafc",
						edgeLabelBackground: "#1e293b",
					},
				});

				// Generate unique ID for this diagram
				const id = `mermaid-${Date.now()}-${Math.random().toString(36).substring(7)}`;

				// Render the diagram
				const { svg: renderedSvg } = await mermaid.default.render(id, chart);

				if (isMounted) {
					setSvg(renderedSvg);
					setIsLoading(false);
				}
			} catch (err) {
				if (isMounted) {
					setError(err instanceof Error ? err.message : "Failed to render diagram");
					setIsLoading(false);
				}
			}
		};

		void renderDiagram();

		return () => {
			isMounted = false;
		};
	}, [chart]);

	if (isLoading) {
		return (
			<div className="flex items-center justify-center p-8 bg-accent/50 rounded-lg border border-border">
				<div className="flex items-center gap-2 text-muted-foreground">
					<svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
						<circle
							className="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							strokeWidth="4"
						/>
						<path
							className="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						/>
					</svg>
					<span className="text-sm">Rendering diagram...</span>
				</div>
			</div>
		);
	}

	if (error) {
		return (
			<div className="p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
				<div className="flex items-start gap-2">
					<svg
						className="h-5 w-5 text-destructive flex-shrink-0 mt-0.5"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						strokeWidth="2"
					>
						<circle cx="12" cy="12" r="10" />
						<line x1="12" y1="8" x2="12" y2="12" />
						<line x1="12" y1="16" x2="12.01" y2="16" />
					</svg>
					<div>
						<p className="text-sm font-medium text-destructive">Failed to render diagram</p>
						<p className="mt-1 text-xs text-muted-foreground">{error}</p>
						<details className="mt-2">
							<summary className="text-xs text-muted-foreground cursor-pointer hover:text-foreground">
								View source
							</summary>
							<pre className="mt-2 p-2 bg-accent rounded text-xs overflow-x-auto">
								<code>{chart}</code>
							</pre>
						</details>
					</div>
				</div>
			</div>
		);
	}

	return (
		<div
			ref={containerRef}
			className="my-4 p-4 bg-accent/30 rounded-lg border border-border overflow-x-auto"
		>
			{svg && (
				<div
					className="flex justify-center [&_svg]:max-w-full [&_svg]:h-auto"
					dangerouslySetInnerHTML={{ __html: svg }}
				/>
			)}
		</div>
	);
}
