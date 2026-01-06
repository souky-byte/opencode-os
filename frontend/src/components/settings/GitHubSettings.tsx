import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  useGetGithubSettings,
  useUpdateGithubSettings,
  useDeleteGithubToken,
  getGetGithubSettingsQueryKey,
} from "@/api/generated/settings/settings";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export function GitHubSettings() {
  const queryClient = useQueryClient();
  const { data: settingsResponse, isLoading } = useGetGithubSettings();
  const updateToken = useUpdateGithubSettings();
  const deleteToken = useDeleteGithubToken();

  const [tokenInput, setTokenInput] = useState("");
  const [isEditing, setIsEditing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const settings = settingsResponse?.data;
  const hasToken = settings?.has_token ?? false;
  const maskedToken = settings?.masked_token;

  const handleSave = () => {
    if (!tokenInput.trim()) {
      setError("Token cannot be empty");
      return;
    }

    setError(null);
    updateToken.mutate(
      { data: { token: tokenInput } },
      {
        onSuccess: () => {
          setTokenInput("");
          setIsEditing(false);
          void queryClient.invalidateQueries({
            queryKey: getGetGithubSettingsQueryKey(),
          });
        },
        onError: () => {
          setError("Failed to save token");
        },
      },
    );
  };

  const handleDelete = () => {
    deleteToken.mutate(undefined, {
      onSuccess: () => {
        void queryClient.invalidateQueries({
          queryKey: getGetGithubSettingsQueryKey(),
        });
      },
      onError: () => {
        setError("Failed to remove token");
      },
    });
  };

  const handleCancel = () => {
    setTokenInput("");
    setIsEditing(false);
    setError(null);
  };

  if (isLoading) {
    return (
      <div className="rounded-lg border p-4">
        <h3 className="font-medium">GitHub Integration</h3>
        <div className="mt-2 text-sm text-muted-foreground">Loading...</div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border p-4">
      <h3 className="font-medium">GitHub Integration</h3>
      <p className="mt-1 text-sm text-muted-foreground">
        Configure your GitHub personal access token for Pull Requests feature.
      </p>

      <div className="mt-4 space-y-4">
        {/* Current status */}
        <div className="flex items-center gap-2">
          <div
            className={`h-2 w-2 rounded-full ${hasToken ? "bg-green-500" : "bg-yellow-500"}`}
          />
          <span className="text-sm">
            {hasToken ? (
              <>
                Token configured:{" "}
                <code className="rounded bg-muted px-1.5 py-0.5 text-xs">
                  {maskedToken}
                </code>
              </>
            ) : (
              "No token configured"
            )}
          </span>
        </div>

        {/* Edit form */}
        {isEditing ? (
          <div className="space-y-3">
            <div>
              <Input
                type="password"
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx"
                value={tokenInput}
                onChange={(e) => setTokenInput(e.target.value)}
                className="font-mono text-sm"
              />
              <p className="mt-1.5 text-xs text-muted-foreground">
                Create a{" "}
                <a
                  href="https://github.com/settings/tokens/new?scopes=repo"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  Personal Access Token
                </a>{" "}
                with <code className="rounded bg-muted px-1">repo</code> scope.
              </p>
            </div>

            {error && <p className="text-sm text-destructive">{error}</p>}

            <div className="flex gap-2">
              <Button
                size="sm"
                onClick={handleSave}
                disabled={updateToken.isPending}
              >
                {updateToken.isPending ? "Saving..." : "Save Token"}
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={handleCancel}
                disabled={updateToken.isPending}
              >
                Cancel
              </Button>
            </div>
          </div>
        ) : (
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={() => setIsEditing(true)}>
              {hasToken ? "Update Token" : "Add Token"}
            </Button>
            {hasToken && (
              <Button
                size="sm"
                variant="ghost"
                className="text-destructive hover:text-destructive"
                onClick={handleDelete}
                disabled={deleteToken.isPending}
              >
                {deleteToken.isPending ? "Removing..." : "Remove"}
              </Button>
            )}
          </div>
        )}

        {/* Info box */}
        {!hasToken && !isEditing && (
          <div className="rounded-md bg-muted/50 p-3">
            <p className="text-sm text-muted-foreground">
              A GitHub token is required to use the Pull Requests feature.
              Alternatively, you can set the{" "}
              <code className="rounded bg-muted px-1">GITHUB_TOKEN</code>{" "}
              environment variable.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
