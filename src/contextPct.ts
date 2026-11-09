export type ContextLevel = 'normal' | 'warn' | 'high';

export function contextLevel(pct: number): ContextLevel {
  if (pct > 85) return 'high';
  if (pct >= 60) return 'warn';
  return 'normal';
}
