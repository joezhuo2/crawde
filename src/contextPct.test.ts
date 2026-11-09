import { describe, expect, it } from 'vitest';
import { contextLevel } from './contextPct';

describe('context level', () => {
  it('stays muted below 60%', () => {
    expect(contextLevel(0)).toBe('normal');
    expect(contextLevel(59)).toBe('normal');
    expect(contextLevel(59.9)).toBe('normal');
  });
  it('turns amber from 60% through 85%', () => {
    expect(contextLevel(60)).toBe('warn');
    expect(contextLevel(85)).toBe('warn');
  });
  it('turns red above 85%', () => {
    expect(contextLevel(86)).toBe('high');
    expect(contextLevel(100)).toBe('high');
  });
});
