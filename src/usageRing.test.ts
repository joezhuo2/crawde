import { describe, expect, it } from 'vitest';
import { formatReset, ringDash, usageLevel, usageText } from './usageRing';

const now = 1_700_000_000_000;
const at = (seconds: number) => now / 1000 + seconds;

describe('usage ring', () => {
  it('fills the arc in proportion to the percentage', () => {
    const circumference = (2 * Math.PI * 10).toFixed(2);
    expect(ringDash(0, 10)).toBe(`0.00 ${circumference}`);
    expect(ringDash(50, 10)).toBe(`31.42 ${circumference}`);
    expect(ringDash(100, 10)).toBe(`${circumference} ${circumference}`);
  });
  it('clamps out-of-range percentages', () => {
    expect(ringDash(-20, 10)).toBe(ringDash(0, 10));
    expect(ringDash(180, 10)).toBe(ringDash(100, 10));
  });
  it('uses the context thresholds for colour levels', () => {
    expect(usageLevel(0)).toBe('normal');
    expect(usageLevel(59)).toBe('normal');
    expect(usageLevel(60)).toBe('warn');
    expect(usageLevel(85)).toBe('warn');
    expect(usageLevel(86)).toBe('high');
  });
  it('formats the time until reset', () => {
    expect(formatReset(at(30), now)).toBe('resets in 1m');
    expect(formatReset(at(14 * 60), now)).toBe('resets in 14m');
    expect(formatReset(at(2 * 3600 + 14 * 60), now)).toBe('resets in 2h 14m');
    expect(formatReset(at(3600), now)).toBe('resets in 1h 0m');
    expect(formatReset(at(3 * 86_400 + 5 * 3600 + 59), now)).toBe('resets in 3d 5h');
  });
  it('reports a reset time that has already passed', () => {
    expect(formatReset(at(0), now)).toBe('reset time has passed');
    expect(formatReset(at(-600), now)).toBe('reset time has passed');
  });
  it('describes a window with and without a reset time', () => {
    expect(usageText('5h', { pct: 37, resetsAt: at(2 * 3600 + 14 * 60) }, now)).toBe('5h plan usage: 37% used - resets in 2h 14m');
    expect(usageText('7d', { pct: 12, resetsAt: null }, now)).toBe('7d plan usage: 12% used');
    expect(usageText('7d', { pct: 12 }, now)).toBe('7d plan usage: 12% used');
  });
});
