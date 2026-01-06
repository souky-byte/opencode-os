import { useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useGetProviders } from "@/api/generated/opencode/opencode";
import {
	getGetRoadmapSettingsQueryKey,
	useGetRoadmapSettings,
	useUpdateRoadmapSettings,
} from "@/api/generated/settings/settings";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

interface ModelSelection {
	provider_id: string;
	model_id: string;
}

const selectClasses = cn(
	"flex h-10 w-full rounded-lg border border-border bg-muted/50 px-3 py-2",
	"text-sm text-foreground",
	"ring-offset-background transition-colors",
	"hover:border-border/80 hover:bg-muted/70",
	"focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:border-primary/50",
	"disabled:cursor-not-allowed disabled:opacity-50",
);

export function RoadmapSettings() {
	const queryClient = useQueryClient();

	const { data: providersResponse, isLoading: isLoadingProviders } = useGetProviders();
	const { data: settingsResponse, isLoading: isLoadingSettings } = useGetRoadmapSettings();

	const [formValue, setFormValue] = useState<ModelSelection>({
		provider_id: "",
		model_id: "",
	});

	const [hasChanges, setHasChanges] = useState(false);

	useEffect(() => {
		if (settingsResponse?.status === 200) {
			const model = settingsResponse.data.config.model;
			if (model) {
				setFormValue({
					provider_id: model.provider_id,
					model_id: model.model_id,
				});
			}
			setHasChanges(false);
		}
	}, [settingsResponse]);

	const updateMutation = useUpdateRoadmapSettings({
		mutation: {
			onSuccess: () => {
				void queryClient.invalidateQueries({ queryKey: getGetRoadmapSettingsQueryKey() });
				setHasChanges(false);
			},
		},
	});

	const providers = providersResponse?.status === 200 ? providersResponse.data.providers : [];
	const selectedProvider = providers.find((p) => p.id === formValue.provider_id);
	const models = selectedProvider?.models ?? [];

	const handleProviderChange = (providerId: string) => {
		const provider = providers.find((p) => p.id === providerId);
		const firstModel = provider?.models[0];
		setFormValue({
			provider_id: providerId,
			model_id: firstModel?.id ?? "",
		});
		setHasChanges(true);
	};

	const handleModelChange = (modelId: string) => {
		setFormValue((prev) => ({
			...prev,
			model_id: modelId,
		}));
		setHasChanges(true);
	};

	const handleSave = () => {
		updateMutation.mutate({
			data: {
				config: {
					model:
						formValue.provider_id && formValue.model_id
							? { provider_id: formValue.provider_id, model_id: formValue.model_id }
							: null,
				},
			},
		});
	};

	const isLoading = isLoadingProviders || isLoadingSettings;

	if (isLoading) {
		return (
			<div className="flex items-center justify-center py-8">
				<div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
				<span className="ml-3 text-sm text-muted-foreground">Loading settings...</span>
			</div>
		);
	}

	if (providers.length === 0) {
		return (
			<div>
				<p className="text-sm text-muted-foreground">
					No AI providers are connected. Connect providers in OpenCode to configure the roadmap
					model.
				</p>
			</div>
		);
	}

	return (
		<div className="space-y-6">
			<div className="rounded-lg border border-border p-4 space-y-3">
				<div>
					<h4 className="font-medium">Generation Model</h4>
					<p className="text-sm text-muted-foreground">
						Select which AI model to use for generating roadmaps
					</p>
				</div>

				<div className="grid grid-cols-2 gap-3">
					<div className="space-y-1.5">
						<label htmlFor="roadmap-provider" className="text-xs font-medium text-muted-foreground">
							Provider
						</label>
						<select
							id="roadmap-provider"
							value={formValue.provider_id}
							onChange={(e) => handleProviderChange(e.target.value)}
							className={selectClasses}
							disabled={updateMutation.isPending}
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
						<label htmlFor="roadmap-model" className="text-xs font-medium text-muted-foreground">
							Model
						</label>
						<select
							id="roadmap-model"
							value={formValue.model_id}
							onChange={(e) => handleModelChange(e.target.value)}
							className={selectClasses}
							disabled={updateMutation.isPending || !formValue.provider_id}
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

			<div className="flex justify-end">
				<Button onClick={handleSave} disabled={!hasChanges || updateMutation.isPending}>
					{updateMutation.isPending ? "Saving..." : "Save Changes"}
				</Button>
			</div>
		</div>
	);
}
