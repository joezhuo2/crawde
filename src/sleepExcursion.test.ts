import { afterEach, expect, it, vi } from 'vitest';
import { scheduleSleepExcursions } from './sleepExcursion';

afterEach(() => vi.useRealTimers());
it.each([0, .999999])('bounds sleeping intervals, duration and attacks (%s)', random => {
  vi.useFakeTimers();
  const wake = vi.fn(), attack = vi.fn();
  const stop = scheduleSleepExcursions(wake, attack, () => random);
  const delay = Math.floor(60_000 + random * 180_000);
  const duration = Math.floor(15_000 + random * 30_000);
  vi.advanceTimersByTime(delay - 1);
  expect(wake).not.toHaveBeenCalled();
  vi.advanceTimersByTime(1);
  expect(wake).toHaveBeenLastCalledWith(true);
  vi.advanceTimersByTime(duration);
  expect(attack).toHaveBeenCalledTimes(1 + Math.floor(random * 3));
  expect(wake).toHaveBeenLastCalledWith(false);
  stop();
  expect(vi.getTimerCount()).toBe(0);
});
it('cancels pending attacks and sleep transitions when interrupted', () => {
  vi.useFakeTimers();
  const wake = vi.fn(), attack = vi.fn();
  const stop = scheduleSleepExcursions(wake, attack, () => 0);
  vi.advanceTimersByTime(60_000);
  stop();
  vi.advanceTimersByTime(300_000);
  expect(attack).not.toHaveBeenCalled();
  expect(wake).toHaveBeenCalledTimes(1);
});
