"use client"

import type React from "react"
import { cn } from "~/lib/utils"
import { LayoutDashboard, Map, Radio, Settings, Sparkles, PanelLeftClose, PanelLeft } from "lucide-react"
import type { View } from "./app-shell"
import { ProjectSelector } from "./project-selector"
import type { Project } from "~/lib/mock-data"
import { Button } from "./ui/button"

interface SidebarProps {
  activeView: View
  onViewChange: (view: View) => void
  projects: Project[]
  selectedProject: Project
  onProjectChange: (project: Project) => void
  collapsed: boolean
  onCollapsedChange: (collapsed: boolean) => void
}

const navItems: { id: View; label: string; icon: React.ReactNode }[] = [
  { id: "kanban", label: "Kanban", icon: <LayoutDashboard className="size-5" /> },
  { id: "roadmap", label: "Roadmap", icon: <Map className="size-5" /> },
  { id: "sessions", label: "Sessions", icon: <Radio className="size-5" /> },
  { id: "settings", label: "Settings", icon: <Settings className="size-5" /> },
]

export function Sidebar({
  activeView,
  onViewChange,
  projects,
  selectedProject,
  onProjectChange,
  collapsed,
  onCollapsedChange,
}: SidebarProps) {
  return (
    <aside
      className={cn(
        "flex flex-col border-r border-border bg-sidebar transition-all duration-200",
        collapsed ? "w-16" : "w-64",
      )}
    >
      {/* Logo */}
      <div className="flex h-14 items-center justify-between border-b border-border px-3">
        <div className="flex items-center gap-2">
          <div className="flex size-8 items-center justify-center rounded-lg bg-primary">
            <Sparkles className="size-4 text-primary-foreground" />
          </div>
          {!collapsed && <span className="text-sm font-semibold tracking-tight">OpenCode</span>}
        </div>
        <Button variant="ghost" size="icon" className="size-8" onClick={() => onCollapsedChange(!collapsed)}>
          {collapsed ? <PanelLeft className="size-4" /> : <PanelLeftClose className="size-4" />}
        </Button>
      </div>

      {/* Project Selector */}
      <div className="border-b border-border p-2">
        <ProjectSelector
          projects={projects}
          selectedProject={selectedProject}
          onProjectChange={onProjectChange}
          collapsed={collapsed}
        />
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col gap-1 p-2">
        {navItems.map((item) => (
          <button
            key={item.id}
            onClick={() => onViewChange(item.id)}
            className={cn(
              "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
              "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
              activeView === item.id ? "bg-sidebar-accent text-sidebar-accent-foreground" : "text-muted-foreground",
              collapsed && "justify-center px-0",
            )}
          >
            {item.icon}
            {!collapsed && <span>{item.label}</span>}
          </button>
        ))}
      </nav>

      {/* Active Sessions Indicator */}
      {!collapsed && selectedProject.activeSessionCount > 0 && (
        <div className="p-2">
          <div className="rounded-lg border border-border bg-card p-3">
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              <div className="size-2 rounded-full bg-primary animate-pulse" />
              <span>
                {selectedProject.activeSessionCount} AI session{selectedProject.activeSessionCount > 1 ? "s" : ""}{" "}
                active
              </span>
            </div>
          </div>
        </div>
      )}
    </aside>
  )
}
