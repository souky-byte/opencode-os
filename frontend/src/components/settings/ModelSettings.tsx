import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import type { ModelSelection, OpenCodeProvider } from "@/api/generated/model";
import { useGetProviders } from "@/api/generated/opencode/opencode";
import {
	getGetPhaseModelsQueryKey,
	useGetPhaseModels,
	useUpdatePhaseModels,
} from "@/api/generated/settings/settings";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type PhaseKey = "planning" | "implementation" | "review" | "fix";

interface PhaseSelection {
	provider_id: string;
	model_id: string;
}

const PHASE_LABELS: Record<PhaseKey, { title: string; description: string }> = {
	planning: {
		title: "Planning",
		description: "Used to analyze tasks and create implementation plans",
	},
	implementation: {
		title: "Implementation",
		description: "Used to write code and implement features",
	},
	review: {
		title: "Review",
		description: "Used to review code changes and provide feedback",
	},
	fix: {
		title: "Fix",
		description: "Used to fix issues found during review",
	},
};

const selectClasses = cn(
	"flex h-10 w-full rounded-lg border border-border bg-muted/50 px-3 py-2",
	"text-sm text-foreground",
	"ring-offset-background transition-colors",
	"hover:border-border/80 hover:bg-muted/70",
	"focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:border-primary/50",
	"disabled:cursor-not-allowed disabled:opacity-50",
);

function PhaseModelSelector({
	phase,
	value,
	providers,
	onChange,
	disabled,
}: {
	phase: PhaseKey;
	value: PhaseSelection;
	providers: OpenCodeProvider[];
	onChange: (value: PhaseSelection) => void;
	disabled?: boolean;
}) {
	const { title, description } = PHASE_LABELS[phase];

	const selectedProvider = providers.find((p) => p.id === value.provider_id);
	const models = selectedProvider?.models ?? [];

	const handleProviderChange = (providerId: string) => {
		const provider = providers.find((p) => p.id === providerId);
		const firstModel = provider?.models[0];
		onChange({
			provider_id: providerId,
			model_id: firstModel?.id ?? "",
		});
	};

	const handleModelChange = (modelId: string) => {
		onChange({
			...value,
			model_id: modelId,
		});
	};

	return (
		<div className="rounded-lg border border-border p-4 space-y-3">
			<div>
				<h4 className="font-medium">{title}</h4>
				<p className="text-sm text-muted-foreground">{description}</p>
			</div>

			<div className="grid grid-cols-2 gap-3">
				<div className="space-y-1.5">
					<label
						htmlFor={`${phase}-provider`}
						className="text-xs font-medium text-muted-foreground"
					>
						Provider
					</label>
					<select
						id={`${phase}-provider`}
						value={value.provider_id}
						onChange={(e) => handleProviderChange(e.target.value)}
						className={selectClasses}
						disabled={disabled}
					>
						<option value="">Select provider...</option>
						{providers.map((provider) => (
							<option key={provider.id} value={provider.id}>
								{provider.name}
							</option>
						))}
					</select>
				</div>

				<div className="space-y-1.5">
					<label htmlFor={`${phase}-model`} className="text-xs font-medium text-muted-foreground">
						Model
					</label>
					<select
						id={`${phase}-model`}
						value={value.model_id}
						onChange={(e) => handleModelChange(e.target.value)}
						className={selectClasses}
						disabled={disabled || !value.provider_id}
					>
						<option value="">Select model...</option>
						{models.map((model) => (
							<option key={model.id} value={model.id}>
								{model.name}
							</option>
						))}
					</select>
				</div>
			</div>
		</div>
	);
}

export function ModelSettings() {
	const queryClient = useQueryClient();

	const { data: providersResponse, isLoading: isLoadingProviders } = useGetProviders();
	const { data: settingsResponse, isLoading: isLoadingSettings } = useGetPhaseModels();

	const [formValues, setFormValues] = useState<Record<PhaseKey, PhaseSelection>>({
		planning: { provider_id: "", model_id: "" },
		implementation: { provider_id: "", model_id: "" },
		review: { provider_id: "", model_id: "" },
		fix: { provider_id: "", model_id: "" },
	});

	const [hasChanges, setHasChanges] = useState(false);

	useEffect(() => {
		if (settingsResponse?.status === 200) {
			const phaseModels = settingsResponse.data.phase_models;
			setFormValues({
				planning: phaseModels.planning ?? { provider_id: "", model_id: "" },
				implementation: phaseModels.implementation ?? { provider_id: "", model_id: "" },
				review: phaseModels.review ?? { provider_id: "", model_id: "" },
				fix: phaseModels.fix ?? { provider_id: "", model_id: "" },
			});
			setHasChanges(false);
		}
	}, [settingsResponse]);

	const updateMutation = useUpdatePhaseModels({
		mutation: {
			onSuccess: () => {
				void queryClient.invalidateQueries({ queryKey: getGetPhaseModelsQueryKey() });
				setHasChanges(false);
			},
		},
	});

	const handlePhaseChange = (phase: PhaseKey, value: PhaseSelection) => {
		setFormValues((prev) => ({
			...prev,
			[phase]: value,
		}));
		setHasChanges(true);
	};

	const handleSave = () => {
		const toModelSelection = (sel: PhaseSelection): ModelSelection | null => {
			if (sel.provider_id && sel.model_id) {
				return { provider_id: sel.provider_id, model_id: sel.model_id };
			}
			return null;
		};

		updateMutation.mutate({
			data: {
				planning: toModelSelection(formValues.planning),
				implementation: toModelSelection(formValues.implementation),
				review: toModelSelection(formValues.review),
				fix: toModelSelection(formValues.fix),
			},
		});
	};

	const isLoading = isLoadingProviders || isLoadingSettings;
	const providers = providersResponse?.status === 200 ? providersResponse.data.providers : [];

	if (isLoading) {
		return (
			<div className="rounded-lg border border-border p-6">
				<div className="flex items-center justify-center py-8">
					<div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
					<span className="ml-3 text-sm text-muted-foreground">Loading model settings...</span>
				</div>
			</div>
		);
	}

	if (providers.length === 0) {
		return (
			<div className="rounded-lg border border-border p-6">
				<h3 className="font-medium">Phase Model Configuration</h3>
				<p className="mt-2 text-sm text-muted-foreground">
					No AI providers are connected. Connect providers in OpenCode to configure models for each
					phase.
				</p>
			</div>
		);
	}

	return (
		<div className="rounded-lg border border-border p-6 space-y-6">
			<div>
				<h3 className="font-medium">Phase Model Configuration</h3>
				<p className="mt-1 text-sm text-muted-foreground">
					Configure which AI model to use for each execution phase.
				</p>
			</div>

			<div className="space-y-4">
				{(["planning", "implementation", "review", "fix"] as const).map((phase) => (
					<PhaseModelSelector
						key={phase}
						phase={phase}
						value={formValues[phase]}
						providers={providers}
						onChange={(value) => handlePhaseChange(phase, value)}
						disabled={updateMutation.isPending}
					/>
				))}
			</div>

			<div className="flex justify-end pt-2">
				<Button onClick={handleSave} disabled={!hasChanges || updateMutation.isPending}>
					{updateMutation.isPending ? "Saving..." : "Save Changes"}
				</Button>
			</div>
		</div>
	);
}
