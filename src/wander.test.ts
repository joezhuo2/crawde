import { expect, it } from 'vitest';
import { nextWalk, walkDelay } from './wander';

it('walks more frequently during work', () => {
  expect(walkDelay(true, () => 0)).toBe(4_000);
  expect(walkDelay(true, () => 1)).toBe(8_000);
  expect(walkDelay(false, () => 0)).toBe(35_000);
});

it('turns back at either edge instead of walking offscreen or getting stuck', () => {
  expect(nextWalk(100, 0, -10, 250, () => 1)).toBe(52);
  expect(nextWalk(-100, 0, -250, 10, () => 0)).toBe(-76);
  expect(nextWalk(0, 0, 0, 200, () => 0)).toBe(24);
  expect(nextWalk(0, 0, -200, 0, () => 1)).toBe(-48);
});

it('stays within 100px of the placed home over repeated walks', () => {
  let position = 20;
  for (let i = 0; i < 1_000; i++) {
    position = nextWalk(position, 20, -12, 180);
    expect(position).toBeGreaterThanOrEqual(-12);
    expect(position).toBeLessThanOrEqual(120);
  }
  expect(nextWalk(0, 0, 0, 0)).toBe(0);
});
