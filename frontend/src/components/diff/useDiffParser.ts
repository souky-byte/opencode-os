import { useMemo } from "react";
import { parse } from "diff2html";
import type { DiffFile } from "diff2html/lib/types";

export type { DiffFile };

export interface ParsedDiff {
	files: DiffFile[];
	totalAdditions: number;
	totalDeletions: number;
	fileCount: number;
}

export function useDiffParser(rawDiff: string | undefined): ParsedDiff | null {
	return useMemo(() => {
		if (!rawDiff || rawDiff.trim() === "") return null;

		try {
			const files = parse(rawDiff);
			return {
				files,
				totalAdditions: files.reduce((sum, f) => sum + f.addedLines, 0),
				totalDeletions: files.reduce((sum, f) => sum + f.deletedLines, 0),
				fileCount: files.length,
			};
		} catch {
			return null;
		}
	}, [rawDiff]);
}

export function getFileDisplayName(file: DiffFile): string {
	// Prefer newName for regular changes, oldName for deletions
	return file.newName || file.oldName || "unknown";
}

export function getFileExtension(file: DiffFile): string {
	const name = getFileDisplayName(file);
	const parts = name.split(".");
	return parts.length > 1 ? parts[parts.length - 1].toUpperCase() : "";
}

export function getFileLanguage(file: DiffFile): string {
	const ext = getFileExtension(file).toLowerCase();
	const languageMap: Record<string, string> = {
		ts: "TypeScript",
		tsx: "TypeScript",
		js: "JavaScript",
		jsx: "JavaScript",
		rs: "Rust",
		py: "Python",
		go: "Go",
		rb: "Ruby",
		java: "Java",
		css: "CSS",
		scss: "SCSS",
		html: "HTML",
		json: "JSON",
		yaml: "YAML",
		yml: "YAML",
		md: "Markdown",
		sql: "SQL",
		sh: "Shell",
		toml: "TOML",
	};
	return languageMap[ext] || ext || "Text";
}
