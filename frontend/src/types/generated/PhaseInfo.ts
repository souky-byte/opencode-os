// Auto-generated TypeScript type

import type { PhaseStatus } from './PhaseStatus';
import type { PhaseSummary } from './PhaseSummary';

export interface PhaseInfo {
  number: number;
  title: string;
  content: string;
  status: PhaseStatus;
  session_id?: string;
  summary?: PhaseSummary;
}
