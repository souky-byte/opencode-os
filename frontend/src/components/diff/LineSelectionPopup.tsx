import { useState, useRef, useEffect } from "react";
import { cn } from "@/lib/utils";

interface LineSelectionPopupProps {
  filePath: string;
  startLine: number;
  endLine: number;
  side: "old" | "new";
  onSave: (content: string) => void;
  onCancel: () => void;
}

export function LineSelectionPopup({
  filePath,
  startLine,
  endLine,
  side,
  onSave,
  onCancel,
}: LineSelectionPopupProps) {
  const [content, setContent] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (content.trim()) {
      onSave(content.trim());
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    } else if (e.key === "Enter" && e.metaKey) {
      e.preventDefault();
      if (content.trim()) {
        onSave(content.trim());
      }
    }
  };

  const lineRange =
    startLine === endLine
      ? `Line ${startLine}`
      : `Lines ${Math.min(startLine, endLine)}-${Math.max(startLine, endLine)}`;

  const fileName = filePath.split("/").pop() ?? filePath;

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <form
        onSubmit={handleSubmit}
        className="bg-[#16161e] border border-white/10 rounded-lg shadow-2xl w-full max-w-md mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="px-4 py-3 border-b border-white/[0.06]">
          <div className="flex items-center gap-2 text-xs">
            <span className="text-white/50">{fileName}</span>
            <span className="text-white/20">•</span>
            <span className="text-white/40">{lineRange}</span>
            <span
              className={cn(
                "px-1.5 py-0.5 rounded text-[10px] font-medium",
                side === "new"
                  ? "bg-emerald-500/10 text-emerald-400/70"
                  : "bg-red-500/10 text-red-400/70",
              )}
            >
              {side === "new" ? "new" : "old"}
            </span>
          </div>
        </div>

        {/* Content */}
        <div className="p-3">
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => setContent(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add your comment..."
            className={cn(
              "w-full min-h-[100px] px-3 py-2.5 rounded-md resize-none",
              "bg-white/[0.03] border border-white/[0.08]",
              "text-sm text-white/90 placeholder:text-white/30",
              "focus:outline-none focus:border-white/20 focus:bg-white/[0.05]",
              "transition-colors",
            )}
          />
        </div>

        {/* Footer */}
        <div className="px-4 py-3 border-t border-white/[0.06] flex items-center justify-between">
          <span className="text-[10px] text-white/30">
            <kbd className="px-1 py-0.5 bg-white/5 rounded border border-white/10">
              ⌘
            </kbd>{" "}
            +{" "}
            <kbd className="px-1 py-0.5 bg-white/5 rounded border border-white/10">
              ↵
            </kbd>{" "}
            to save
          </span>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onCancel}
              className={cn(
                "px-3 py-1.5 text-xs font-medium rounded-md",
                "text-white/50 hover:text-white/70 hover:bg-white/5",
                "transition-colors",
              )}
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={!content.trim()}
              className={cn(
                "px-3 py-1.5 text-xs font-medium rounded-md",
                "bg-blue-500/80 text-white",
                "hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed",
                "transition-colors",
              )}
            >
              Save comment
            </button>
          </div>
        </div>
      </form>
    </div>
  );
}
