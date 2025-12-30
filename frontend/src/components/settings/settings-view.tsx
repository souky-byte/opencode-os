"use client"

import { useState } from "react"
import { Button } from "~/components/ui/button"
import { Input } from "~/components/ui/input"
import { Label } from "~/components/ui/label"
import { Switch } from "~/components/ui/switch"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select"
import { Separator } from "~/components/ui/separator"
import { Save } from "lucide-react"

export function SettingsView() {
  const [requirePlanApproval, setRequirePlanApproval] = useState(false)
  const [autoStartDevServer, setAutoStartDevServer] = useState(true)

  return (
    <div className="h-full p-6 max-w-2xl">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="text-sm text-muted-foreground">Configure OpenCode Studio behavior</p>
      </div>

      <div className="space-y-8">
        {/* Project Settings */}
        <section>
          <h2 className="text-lg font-semibold mb-4">Project</h2>
          <div className="space-y-4">
            <div className="grid gap-2">
              <Label htmlFor="project-name">Project Name</Label>
              <Input id="project-name" defaultValue="my-app" />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="repository">Repository</Label>
              <Input id="repository" defaultValue="git@github.com:user/my-app.git" className="font-mono text-sm" />
            </div>
          </div>
        </section>

        <Separator />

        {/* Kanban Settings */}
        <section>
          <h2 className="text-lg font-semibold mb-4">Kanban</h2>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label>Require Plan Approval</Label>
                <p className="text-xs text-muted-foreground">Human must approve plan before implementation starts</p>
              </div>
              <Switch checked={requirePlanApproval} onCheckedChange={setRequirePlanApproval} />
            </div>
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label>Auto Start Dev Server</Label>
                <p className="text-xs text-muted-foreground">Automatically start dev server when task enters REVIEW</p>
              </div>
              <Switch checked={autoStartDevServer} onCheckedChange={setAutoStartDevServer} />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="parallel-tasks">Parallel Tasks Limit</Label>
              <Input id="parallel-tasks" type="number" defaultValue="5" className="w-24" />
            </div>
          </div>
        </section>

        <Separator />

        {/* AI Models */}
        <section>
          <h2 className="text-lg font-semibold mb-4">AI Models</h2>
          <div className="space-y-4">
            <div className="grid gap-2">
              <Label htmlFor="planning-model">Planning Model</Label>
              <Select defaultValue="claude-sonnet">
                <SelectTrigger id="planning-model">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="claude-sonnet">claude-sonnet-4-20250514</SelectItem>
                  <SelectItem value="claude-opus">claude-opus-4-20250514</SelectItem>
                  <SelectItem value="gpt-4">gpt-4-turbo</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="impl-model">Implementation Model</Label>
              <Select defaultValue="claude-sonnet">
                <SelectTrigger id="impl-model">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="claude-sonnet">claude-sonnet-4-20250514</SelectItem>
                  <SelectItem value="claude-opus">claude-opus-4-20250514</SelectItem>
                  <SelectItem value="gpt-4">gpt-4-turbo</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="review-model">Review Model</Label>
              <Select defaultValue="claude-sonnet">
                <SelectTrigger id="review-model">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="claude-sonnet">claude-sonnet-4-20250514</SelectItem>
                  <SelectItem value="claude-opus">claude-opus-4-20250514</SelectItem>
                  <SelectItem value="gpt-4">gpt-4-turbo</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </section>

        <Separator />

        {/* VCS Settings */}
        <section>
          <h2 className="text-lg font-semibold mb-4">Version Control</h2>
          <div className="space-y-4">
            <div className="grid gap-2">
              <Label htmlFor="vcs-backend">VCS Backend</Label>
              <Select defaultValue="jj">
                <SelectTrigger id="vcs-backend">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="jj">Jujutsu (jj)</SelectItem>
                  <SelectItem value="git">Git Worktrees</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-2">
              <Label htmlFor="workspace-path">Workspace Base Path</Label>
              <Input id="workspace-path" defaultValue="../.workspaces" className="font-mono text-sm" />
            </div>
          </div>
        </section>

        <div className="pt-4">
          <Button className="gap-2">
            <Save className="size-4" />
            Save Changes
          </Button>
        </div>
      </div>
    </div>
  )
}
