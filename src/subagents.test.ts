import { describe, expect, it } from 'vitest';
import { agentExtraHeight, agentListHeight, agentStateLabel, agentSummary } from './subagents';
import type { Subagent } from './types';

const agent: Subagent = { id: 'a', name: 'explorer', task: 'Map the repo', state: 'working', tool: 'Grep' };

describe('subagent details', () => {
  it('pluralises the summary line', () => {
    expect(agentSummary(1)).toBe('🤝 1 subagent running');
    expect(agentSummary(2)).toBe('🤝 2 subagents running');
  });

  it('describes what each agent is doing', () => {
    expect(agentStateLabel(agent)).toBe('Using Grep');
    expect(agentStateLabel({ ...agent, tool: '' })).toBe('Working');
    expect(agentStateLabel({ ...agent, state: 'thinking', tool: '' })).toBe('Thinking');
    expect(agentStateLabel({ ...agent, state: 'approval' })).toBe('Waiting to run Grep');
    expect(agentStateLabel({ ...agent, state: 'approval', tool: '' })).toBe('Waiting for approval');
    expect(agentStateLabel({ ...agent, state: 'starting', tool: '' })).toBe('Starting');
  });

  it('caps the list height at three rows and scrolls beyond that', () => {
    expect(agentListHeight(0)).toBe(0);
    expect(agentListHeight(1)).toBe(32);
    expect(agentListHeight(3)).toBe(96);
    expect(agentListHeight(9)).toBe(96);
  });

  it('adds spacing only when there are agents to show', () => {
    expect(agentExtraHeight(0)).toBe(0);
    expect(agentExtraHeight(2)).toBe(72);
    expect(agentExtraHeight(9)).toBe(104);
  });
});
