import type { ErrorInfo, ReactNode } from "react";
import { Component } from "react";
import { Button } from "@/components/ui/button";

interface ErrorBoundaryProps {
	children: ReactNode;
	fallback?: ReactNode;
}

interface ErrorBoundaryState {
	hasError: boolean;
	error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
	constructor(props: ErrorBoundaryProps) {
		super(props);
		this.state = { hasError: false, error: null };
	}

	static getDerivedStateFromError(error: Error): ErrorBoundaryState {
		return { hasError: true, error };
	}

	componentDidCatch(_error: Error, _errorInfo: ErrorInfo) {}

	handleReset = () => {
		this.setState({ hasError: false, error: null });
	};

	render() {
		if (this.state.hasError) {
			if (this.props.fallback) {
				return this.props.fallback;
			}

			return (
				<div className="flex h-full min-h-[200px] flex-col items-center justify-center gap-4 p-6">
					<div className="text-center">
						<h2 className="text-lg font-semibold text-destructive">Something went wrong</h2>
						<p className="mt-2 text-sm text-muted-foreground">
							{this.state.error?.message ?? "An unexpected error occurred"}
						</p>
					</div>
					<Button variant="outline" onClick={this.handleReset}>
						Try Again
					</Button>
				</div>
			);
		}

		return this.props.children;
	}
}
