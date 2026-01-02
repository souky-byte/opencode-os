// Auto-generated TypeScript type

import type { PhaseInfo } from './PhaseInfo';

export interface PhasesResponse {
  is_multi_phase: boolean;
  total_phases: number;
  current_phase: number | null;
  phases: PhaseInfo[];
}
