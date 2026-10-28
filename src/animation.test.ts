import { describe, expect, it } from 'vitest';
import { animations, attackEffect, framePosition } from './animation';

describe('sprite cells', () => {
  it('moves to the next row without using empty trailing cells', () => {
    expect(framePosition(4, 4, 3)).toBe('-48px -276px');
    expect(animations.run.frames).toBe(6);
    expect(animations.ability.frames).toBe(10);
  });
});

it('aligns all three FX cells with the strike frames in CrabComplete.gif', () => {
  expect(Array.from({ length: 10 }, (_, frame) => attackEffect('attackA', frame)?.frame ?? null))
    .toEqual([null, null, null, null, null, null, 0, 1, 2, null]);
  expect(Array.from({ length: 6 }, (_, frame) => attackEffect('attackB', frame)?.frame ?? null))
    .toEqual([null, null, 0, 1, 2, null]);
  expect(Array.from({ length: 11 }, (_, frame) => attackEffect('attackC', frame)?.frame ?? null))
    .toEqual([null, null, null, null, 0, 1, 2, 3, 4, 5, null]);
  expect(attackEffect('ability', 6)).toBeNull();
  expect(attackEffect('idle', 2)).toBeNull();
  expect(attackEffect('attackA', 6)?.image).not.toBe(attackEffect('attackB', 2)?.image);
});

import { clickAnimation } from './animation';

it('chooses each click animation with equal probability', () => {
  expect([0, .25, .5, .75, .999999].map(value => clickAnimation(() => value))).toEqual(['ability', 'attackA', 'attackB', 'attackC', 'attackC']);
});

import { randomAttack } from './animation';

it('chooses attacks A, B and C with equal probability', () => {
  expect([0, .34, .67, .999999].map(value => randomAttack(() => value))).toEqual(['attackA', 'attackB', 'attackC', 'attackC']);
});
