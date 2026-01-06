import { useEffect, useRef, useState } from "react";

interface MermaidDiagramProps {
	chart: string;
}

export function MermaidDiagram({ chart }: MermaidDiagramProps) {
	const containerRef = useRef<HTMLDivElement>(null);
	const [svg, setSvg] = useState<string | null>(null);
	const [error, setError] = useState<string | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [showSource, setShowSource] = useState(false);

	useEffect(() => {
		let isMounted = true;

		const renderDiagram = async () => {
			try {
				setIsLoading(true);
				setError(null);

				const mermaid = await import("mermaid");

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

				const id = `mermaid-${Date.now()}-${Math.random().toString(36).substring(7)}`;
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
			<div className="my-4 rounded-lg border border-border overflow-hidden">
				<div className="flex items-center justify-between px-3 py-2 bg-accent/50 border-b border-border">
					<div className="flex items-center gap-2 text-xs text-muted-foreground">
						<svg
							className="h-4 w-4"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							strokeWidth="2"
						>
							<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
							<polyline points="3.27 6.96 12 12.01 20.73 6.96" />
							<line x1="12" y1="22.08" x2="12" y2="12" />
						</svg>
						<span>Mermaid Diagram</span>
						<span className="text-amber-500">(parse error)</span>
					</div>
					<button
						type="button"
						onClick={() => setShowSource(!showSource)}
						className="text-xs text-muted-foreground hover:text-foreground"
					>
						{showSource ? "Hide error" : "Show error"}
					</button>
				</div>
				{showSource && (
					<div className="px-3 py-2 bg-amber-500/10 border-b border-border text-xs text-amber-500">
						{error}
					</div>
				)}
				<pre className="p-3 bg-accent/30 overflow-x-auto text-xs">
					<code className="text-muted-foreground">{chart}</code>
				</pre>
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
