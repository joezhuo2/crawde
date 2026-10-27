import { describe, expect, it } from 'vitest';
import { animations, framePosition } from './animation';

describe('sprite cells', () => {
  it('moves to the next row without using empty trailing cells', () => {
    expect(framePosition(4, 4, 3)).toBe('-48px -276px');
    expect(animations.run.frames).toBe(6);
    expect(animations.ability.frames).toBe(10);
  });
});

import { clickAnimation } from './animation';

it('chooses each click animation with equal probability', () => {
  expect([0, .333333, 1 / 3, .666666, 2 / 3, .999999].map(value => clickAnimation(() => value))).toEqual(['ability', 'ability', 'attackA', 'attackA', 'attackB', 'attackB']);
});
