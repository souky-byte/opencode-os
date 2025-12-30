"use client"

import { Button } from "~/components/ui/button"
import { Bell, Plus, Search } from "lucide-react"
import { Input } from "~/components/ui/input"
import type { Project } from "~/lib/mock-data"
import { Badge } from "./ui/badge"

interface HeaderProps {
  project: Project
}

export function Header({ project }: HeaderProps) {
  return (
    <header className="flex h-14 items-center justify-between border-b border-border bg-background px-4">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 text-sm">
          <Badge variant="secondary" className="font-mono text-xs">
            {project.defaultBranch}
          </Badge>
          <span className="text-muted-foreground">•</span>
          <span className="text-muted-foreground text-xs">
            {project.vcsBackend === "jj" ? "jj workspace" : "git repo"}
          </span>
        </div>
      </div>
      <div className="flex items-center gap-3">
        <div className="relative hidden md:block">
          <Search className="absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input placeholder="Search tasks..." className="w-64 pl-9 h-8 bg-secondary border-0" />
        </div>
        <Button size="sm" className="gap-1.5">
          <Plus className="size-4" />
          <span className="hidden sm:inline">New Task</span>
        </Button>
        <Button variant="ghost" size="icon" className="relative">
          <Bell className="size-4" />
          <span className="absolute -right-0.5 -top-0.5 size-2 rounded-full bg-primary" />
        </Button>
      </div>
    </header>
  )
}
