// Offsets are relative to the crab's layout anchor, never the native window.
export function nextWalk(current: number, home: number, min: number, max: number, random = Math.random) {
  const low = Math.max(min, home - 100);
  const high = Math.max(low, Math.min(max, home + 100));
  const step = (random() < .5 ? -1 : 1) * (24 + random() * 24);
  const clamp = (value: number) => Math.max(low, Math.min(high, value));
  const target = clamp(current + step);
  return Math.abs(target - current) < 8 ? clamp(current - step) : target;
}

export function walkDelay(working: boolean, random = Math.random) {
  return working ? 4_000 + random() * 4_000 : 35_000 + random() * 25_000;
}
