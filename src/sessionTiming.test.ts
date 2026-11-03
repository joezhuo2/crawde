import { describe, expect, it } from 'vitest';
import { formatDuration, sessionTiming } from './sessionTiming';
import type { Session } from './types';

const session: Session = { id: 's', cwd: 'C:\\work', project: 'work', title: 'A task', status: 'working', activity: '📚 Reading', tool: 'Read', agents: [], updatedAt: 100_000, lastPromptAt: 1, finishedAt: null, workingMs: 0, waitingMs: 0, statusSince: 100_000 };

describe('session timing', () => {
  it('formats durations compactly', () => {
    expect(formatDuration(-5)).toBe('0s');
    expect(formatDuration(999)).toBe('0s');
    expect(formatDuration(18_000)).toBe('18s');
    expect(formatDuration(134_000)).toBe('2m 14s');
    expect(formatDuration(3_900_000)).toBe('1h 5m');
  });
  it('adds the running working segment to the stored total', () => {
    const line = sessionTiming({ ...session, workingMs: 120_000, statusSince: 100_000 }, 114_000);
    expect(line).toBe('2m 14s working · active 14s ago');
  });
  it('shows waiting time and grows it while approval is pending', () => {
    const waiting = { ...session, status: 'approval' as const, workingMs: 5_000, waitingMs: 10_000, statusSince: 100_000 };
    expect(sessionTiming(waiting, 108_000)).toBe('5s working · 18s waiting · active 8s ago');
  });
  it('stops counting once the session is done', () => {
    const done = { ...session, status: 'done' as const, workingMs: 60_000, statusSince: 100_000 };
    expect(sessionTiming(done, 160_000)).toBe('1m 0s working · active 1m 0s ago');
  });
});
