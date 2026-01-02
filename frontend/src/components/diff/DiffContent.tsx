import { useMemo } from "react";
import { html } from "diff2html";
import type { DiffFile } from "diff2html/lib/types";
import { ColorSchemeType } from "diff2html/lib/types";
import "./diff-overrides.css";

interface DiffContentProps {
	file: DiffFile;
}

export function DiffContent({ file }: DiffContentProps) {
	const diffHtml = useMemo(() => {
		return html([file], {
			outputFormat: "side-by-side",
			drawFileList: false,
			matching: "lines",
			colorScheme: ColorSchemeType.DARK,
		});
	}, [file]);

	return (
		<div
			className="diff-content overflow-x-auto"
			// biome-ignore lint/security/noDangerouslySetInnerHtml: diff2html generates safe HTML
			dangerouslySetInnerHTML={{ __html: diffHtml }}
		/>
	);
}
