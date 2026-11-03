import type { Subagent } from './types';

export const AGENT_ROW_HEIGHT = 32;
export const AGENT_VISIBLE_ROWS = 3;
export const AGENT_PANEL_GAP = 8;

export function agentListHeight(count: number) {
  return count > 0 ? Math.min(count, AGENT_VISIBLE_ROWS) * AGENT_ROW_HEIGHT : 0;
}

export function agentExtraHeight(count: number) {
  return count > 0 ? agentListHeight(count) + AGENT_PANEL_GAP : 0;
}

export function agentSummary(count: number) {
  return `🤝 ${count} subagent${count === 1 ? '' : 's'} running`;
}

export function agentStateLabel(agent: Subagent) {
  switch (agent.state) {
    case 'working': return agent.tool ? `Using ${agent.tool}` : 'Working';
    case 'approval': return agent.tool ? `Waiting to run ${agent.tool}` : 'Waiting for approval';
    case 'thinking': return 'Thinking';
    default: return 'Starting';
  }
}
