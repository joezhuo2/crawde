import { describe, expect, it } from 'vitest';
import { shouldSleep, SLEEP_AFTER_MS } from './sleep';
import type { Session } from './types';

const now = 2_000_000;
const session: Session = { id: 's', cwd: 'C:\\work', project: 'work', title: 'A task', status: 'working', activity: '📚 Reading', tool: 'Read', agents: [], updatedAt: now, lastPromptAt: 1, finishedAt: null };

describe('sleep state', () => {
  it('wakes from tool activity even if process detection misses Claude', () => {
    expect(shouldSleep([session], false, 0, now, now)).toBe(false);
  });
  it('stays awake when work continues beyond ten minutes after the prompt', () => {
    expect(shouldSleep([session], true, 1, now, now)).toBe(false);
  });
  it('sleeps after ten minutes without activity', () => {
    expect(shouldSleep([session], true, 1, now, now + SLEEP_AFTER_MS)).toBe(true);
  });
  it('sleeps when Claude closes and all work has finished', () => {
    expect(shouldSleep([{ ...session, status: 'done' }], false, now, now, now)).toBe(true);
  });
  it('uses global activity when cards are dismissed', () => {
    expect(shouldSleep([], true, 1, now, now)).toBe(false);
  });
});
