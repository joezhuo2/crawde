import { describe, expect, it } from 'vitest';
import { animations, framePosition } from './animation';

describe('sprite cells', () => {
  it('moves to the next row without using empty trailing cells', () => {
    expect(framePosition(4, 4, 3)).toBe('-48px -252px');
    expect(animations.run.frames).toBe(6);
    expect(animations.ability.frames).toBe(10);
  });
});
