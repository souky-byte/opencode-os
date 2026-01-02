import { useMemo, useCallback, useEffect, useRef } from "react";
import { html } from "diff2html";
import type { DiffFile } from "diff2html/lib/types";
import { ColorSchemeType } from "diff2html/lib/types";
import "./diff-overrides.css";

export interface LineSelection {
  startLine: number;
  endLine: number;
  side: "old" | "new";
}

interface DiffContentProps {
  file: DiffFile;
  selection?: LineSelection | null;
  onLineClick?: (
    lineNumber: number,
    side: "old" | "new",
    shiftKey: boolean,
  ) => void;
  commentedLines?: Array<{ line: number; side: "old" | "new" }>;
}

export function DiffContent({
  file,
  selection,
  onLineClick,
  commentedLines = [],
}: DiffContentProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  const diffHtml = useMemo(() => {
    return html([file], {
      outputFormat: "side-by-side",
      drawFileList: false,
      matching: "lines",
      colorScheme: ColorSchemeType.DARK,
    });
  }, [file]);

  // Handle click on diff content - only right (new) side, click anywhere on row
  const handleClick = useCallback(
    (e: React.MouseEvent) => {
      if (!onLineClick) return;

      const target = e.target as HTMLElement;

      // Find the row
      const row = target.closest("tr");
      if (!row) return;

      // Find which side panel we're in
      const sideDiff = row.closest(".d2h-file-side-diff");
      if (!sideDiff) return;

      // Only allow selection on the right (new) side
      const wrapper = sideDiff.closest(".d2h-files-diff");
      if (!wrapper) return;

      const panels = wrapper.querySelectorAll(".d2h-file-side-diff");
      const isNewSide = panels[1] === sideDiff;

      // Ignore clicks on the old (left) side
      if (!isNewSide) return;

      // Get line number from the row
      const lineNumCell = row.querySelector(".d2h-code-side-linenumber");
      if (!lineNumCell) return;

      const lineNum = lineNumCell.textContent?.trim();
      if (!lineNum || lineNum === "...") return;

      const lineNumber = parseInt(lineNum, 10);
      if (isNaN(lineNumber)) return;

      onLineClick(lineNumber, "new", e.shiftKey);
    },
    [onLineClick],
  );

  // Apply selection highlighting via DOM manipulation
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // Clear previous selection
    container.querySelectorAll(".line-selected").forEach((el) => {
      el.classList.remove("line-selected");
    });

    // Clear comment indicators
    container.querySelectorAll(".has-comment").forEach((el) => {
      el.classList.remove("has-comment");
    });

    // Apply new selection
    if (selection) {
      const { startLine, endLine, side } = selection;
      const minLine = Math.min(startLine, endLine);
      const maxLine = Math.max(startLine, endLine);

      // Find the correct panel based on side
      const panels = container.querySelectorAll(".d2h-file-side-diff");
      const targetPanel = side === "old" ? panels[0] : panels[1];

      if (targetPanel) {
        const rows = targetPanel.querySelectorAll("tr");
        rows.forEach((row) => {
          const lineNumCell = row.querySelector(".d2h-code-side-linenumber");
          if (!lineNumCell) return;

          const lineNum = parseInt(lineNumCell.textContent?.trim() || "", 10);
          if (!isNaN(lineNum) && lineNum >= minLine && lineNum <= maxLine) {
            row.classList.add("line-selected");
          }
        });
      }
    }

    // Apply comment indicators
    commentedLines.forEach(({ line, side }) => {
      const panels = container.querySelectorAll(".d2h-file-side-diff");
      const targetPanel = side === "old" ? panels[0] : panels[1];

      if (targetPanel) {
        const rows = targetPanel.querySelectorAll("tr");
        rows.forEach((row) => {
          const lineNumCell = row.querySelector(".d2h-code-side-linenumber");
          if (!lineNumCell) return;

          const lineNum = parseInt(lineNumCell.textContent?.trim() || "", 10);
          if (lineNum === line) {
            lineNumCell.classList.add("has-comment");
          }
        });
      }
    });
  }, [selection, commentedLines, diffHtml]);

  return (
    <div
      ref={containerRef}
      className="diff-content overflow-x-auto"
      onClick={handleClick}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: diff2html generates safe HTML
      dangerouslySetInnerHTML={{ __html: diffHtml }}
    />
  );
}
