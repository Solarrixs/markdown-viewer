import type { AnnotationRecord, ReviewStatusRecord } from './stores';

/**
 * Generate structured markdown feedback from annotations and review statuses.
 * This is sent to Claude Code sessions as review feedback.
 */
export function generateFeedbackMarkdown(
  annotations: AnnotationRecord[],
  reviewStatuses: ReviewStatusRecord[],
): string {
  const lines: string[] = [];

  lines.push('# Code Review Feedback');
  lines.push('');

  // Group annotations by file path
  const byFile = new Map<string, AnnotationRecord[]>();
  for (const a of annotations) {
    const existing = byFile.get(a.file_path) ?? [];
    existing.push(a);
    byFile.set(a.file_path, existing);
  }

  if (byFile.size > 0) {
    lines.push('## Inline Annotations');
    lines.push('');

    for (const [filePath, fileAnnotations] of byFile) {
      lines.push(`### ${filePath}`);
      lines.push('');
      for (const a of fileAnnotations) {
        lines.push(`- **Line ${a.line_number}**: ${a.annotation_text}`);
        if (a.commit_hash) {
          lines.push(`  _(commit: ${a.commit_hash.slice(0, 7)})_`);
        }
      }
      lines.push('');
    }
  }

  // Add commits that need changes
  const needsChanges = reviewStatuses.filter(r => r.status === 'needs_changes');
  if (needsChanges.length > 0) {
    lines.push('## Commits Needing Changes');
    lines.push('');
    for (const r of needsChanges) {
      lines.push(`- **${r.commit_hash.slice(0, 7)}**: ${r.notes ?? 'Needs revision'}`);
    }
    lines.push('');
  }

  // Summary
  const reviewed = reviewStatuses.filter(r => r.status === 'reviewed').length;
  const total = reviewStatuses.length;
  if (total > 0) {
    lines.push('## Summary');
    lines.push('');
    lines.push(`- ${reviewed}/${total} commits reviewed`);
    lines.push(`- ${needsChanges.length} commits need changes`);
    lines.push(`- ${annotations.length} inline annotations`);
  }

  return lines.join('\n');
}
