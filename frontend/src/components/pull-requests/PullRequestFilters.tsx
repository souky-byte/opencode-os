import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  usePullRequestStore,
  type PrSortField,
  type PrSortOrder,
} from "@/stores/usePullRequestStore";
import type { PrState } from "@/api/generated/model";
import { RotateCcw, ArrowUpDown } from "lucide-react";

type StateFilter = PrState | "all";

function PullRequestFilters() {
  const { filters, sort, setFilter, setSort, resetFilters } =
    usePullRequestStore();

  return (
    <div className="flex items-center gap-2 p-2 border-b bg-muted/30">
      {/* State filter */}
      <Select
        value={filters.state}
        onValueChange={(value: StateFilter) => setFilter("state", value)}
      >
        <SelectTrigger className="w-24 h-7 text-xs">
          <SelectValue placeholder="State" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="open">Open</SelectItem>
          <SelectItem value="closed">Closed</SelectItem>
          <SelectItem value="all">All</SelectItem>
        </SelectContent>
      </Select>

      {/* Sort */}
      <Select
        value={`${sort.field}-${sort.order}`}
        onValueChange={(value: string) => {
          const [field, order] = value.split("-") as [PrSortField, PrSortOrder];
          setSort(field, order);
        }}
      >
        <SelectTrigger className="w-32 h-7 text-xs">
          <ArrowUpDown className="w-3 h-3 mr-1" />
          <SelectValue placeholder="Sort" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="updated-desc">Recently updated</SelectItem>
          <SelectItem value="updated-asc">Least updated</SelectItem>
          <SelectItem value="created-desc">Newest</SelectItem>
          <SelectItem value="created-asc">Oldest</SelectItem>
        </SelectContent>
      </Select>

      {/* Reset */}
      <Button
        variant="ghost"
        size="sm"
        className="h-7 px-2 text-xs"
        onClick={resetFilters}
      >
        <RotateCcw className="w-3 h-3" />
      </Button>
    </div>
  );
}

export { PullRequestFilters };
