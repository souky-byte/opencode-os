"use client"

import { useState } from "react"
import { mockRoadmapItems, type RoadmapItem } from "~/lib/mock-data"
import { RoadmapCard } from "./roadmap-card"
import { RoadmapDetailPanel } from "./roadmap-detail-panel"
import { Button } from "~/components/ui/button"
import { Plus, Sparkles } from "lucide-react"

export function RoadmapView() {
  const [selectedItem, setSelectedItem] = useState<RoadmapItem | null>(null)

  const q1Items = mockRoadmapItems.filter((item) => item.quarter === "Q1 2025")
  const q2Items = mockRoadmapItems.filter((item) => item.quarter === "Q2 2025")

  return (
    <div className="h-full p-6">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Roadmap</h1>
          <p className="text-sm text-muted-foreground">Produktová vrstva • {mockRoadmapItems.length} položek</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" className="gap-2 bg-transparent">
            <Sparkles className="size-4" />
            AI Generate
          </Button>
          <Button className="gap-2">
            <Plus className="size-4" />
            New Item
          </Button>
        </div>
      </div>

      <div className={selectedItem ? "mr-[540px] transition-all" : "transition-all"}>
        <div className="space-y-8">
          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <span className="size-3 rounded-full bg-primary" />
              Q1 2025
            </h2>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {q1Items.map((item) => (
                <RoadmapCard key={item.id} item={item} onClick={() => setSelectedItem(item)} />
              ))}
            </div>
          </section>

          <section>
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <span className="size-3 rounded-full bg-chart-2" />
              Q2 2025
            </h2>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {q2Items.map((item) => (
                <RoadmapCard key={item.id} item={item} onClick={() => setSelectedItem(item)} />
              ))}
            </div>
          </section>
        </div>
      </div>

      {selectedItem && <RoadmapDetailPanel item={selectedItem} onClose={() => setSelectedItem(null)} />}
    </div>
  )
}
