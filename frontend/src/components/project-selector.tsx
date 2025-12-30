"use client"

import { useState } from "react"
import { cn } from "~/lib/utils"
import type { Project } from "~/lib/mock-data"
import { Check, ChevronsUpDown, GitBranch, Plus } from "lucide-react"
import { Button } from "~/components/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "~/components/ui/command"
import { Popover, PopoverContent, PopoverTrigger } from "~/components/ui/popover"

interface ProjectSelectorProps {
  projects: Project[]
  selectedProject: Project
  onProjectChange: (project: Project) => void
  collapsed?: boolean
}

export function ProjectSelector({
  projects,
  selectedProject,
  onProjectChange,
  collapsed = false,
}: ProjectSelectorProps) {
  const [open, setOpen] = useState(false)

  const formatLastActivity = (date: string) => {
    const diff = Date.now() - new Date(date).getTime()
    const hours = Math.floor(diff / (1000 * 60 * 60))
    if (hours < 1) return "Active now"
    if (hours < 24) return `${hours}h ago`
    const days = Math.floor(hours / 24)
    return `${days}d ago`
  }

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          role="combobox"
          aria-expanded={open}
          className={cn(
            "w-full justify-start gap-3 px-3 py-6 hover:bg-sidebar-accent",
            collapsed && "justify-center px-0",
          )}
        >
          <div
            className="flex size-8 shrink-0 items-center justify-center rounded-lg text-white text-sm font-semibold"
            style={{ backgroundColor: selectedProject.color }}
          >
            {selectedProject.name.charAt(0)}
          </div>
          {!collapsed && (
            <>
              <div className="flex flex-1 flex-col items-start text-left">
                <span className="text-sm font-medium truncate max-w-[120px]">{selectedProject.name}</span>
                <span className="text-xs text-muted-foreground flex items-center gap-1">
                  <GitBranch className="size-3" />
                  {selectedProject.defaultBranch}
                </span>
              </div>
              <ChevronsUpDown className="size-4 shrink-0 text-muted-foreground" />
            </>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-72 p-0" align="start" side="right">
        <Command>
          <CommandInput placeholder="Search projects..." />
          <CommandList>
            <CommandEmpty>No project found.</CommandEmpty>
            <CommandGroup heading="Projects">
              {projects.map((project) => (
                <CommandItem
                  key={project.id}
                  value={project.name}
                  onSelect={() => {
                    onProjectChange(project)
                    setOpen(false)
                  }}
                  className="flex items-center gap-3 py-3"
                >
                  <div
                    className="flex size-8 shrink-0 items-center justify-center rounded-lg text-white text-sm font-semibold"
                    style={{ backgroundColor: project.color }}
                  >
                    {project.name.charAt(0)}
                  </div>
                  <div className="flex flex-1 flex-col">
                    <span className="text-sm font-medium">{project.name}</span>
                    <span className="text-xs text-muted-foreground line-clamp-1">{project.description}</span>
                  </div>
                  <div className="flex flex-col items-end gap-1">
                    {project.activeSessionCount > 0 && (
                      <span className="flex items-center gap-1 text-xs text-primary">
                        <span className="size-1.5 rounded-full bg-primary animate-pulse" />
                        {project.activeSessionCount} active
                      </span>
                    )}
                    <span className="text-xs text-muted-foreground">{formatLastActivity(project.lastActivity)}</span>
                  </div>
                  {selectedProject.id === project.id && <Check className="size-4 text-primary" />}
                </CommandItem>
              ))}
            </CommandGroup>
            <CommandSeparator />
            <CommandGroup>
              <CommandItem className="flex items-center gap-2 py-2">
                <Plus className="size-4" />
                <span>Create new project</span>
              </CommandItem>
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
