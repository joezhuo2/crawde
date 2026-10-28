// Offsets are relative to the crab's layout anchor, never the native window.
export function nextWalk(current: number, home: number, min: number, max: number, random = Math.random) {
  const low = Math.max(min, home - 100);
  const high = Math.max(low, Math.min(max, home + 100));
  const step = (random() < .5 ? -1 : 1) * (24 + random() * 24);
  const clamp = (value: number) => Math.max(low, Math.min(high, value));
  const target = clamp(current + step);
  return Math.abs(target - current) < 8 ? clamp(current - step) : target;
}

export type WalkMode = 'idle' | 'working' | 'exploring';

export function walkDelay(mode: WalkMode, random = Math.random) {
  if (mode === 'exploring') return 3_000 + random() * 4_000;
  return mode === 'working' ? 1_000 + random() * 2_000 : 35_000 + random() * 25_000;
}
