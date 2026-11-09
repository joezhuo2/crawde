import { contextLevel, type ContextLevel } from './contextPct';
import type { Usage, UsageWindow } from './types';

export function ringDash(pct: number, radius: number): string {
  const circumference = 2 * Math.PI * radius;
  const filled = circumference * Math.min(100, Math.max(0, pct)) / 100;
  return `${filled.toFixed(2)} ${circumference.toFixed(2)}`;
}

export function usageLevel(pct: number): ContextLevel {
  return contextLevel(pct);
}

export function hasUsage(usage?: Usage | null): usage is Usage {
  return Boolean(usage && (usage.fiveHour || usage.sevenDay));
}

export function formatReset(resetsAt: number, now: number): string {
  const minutes = Math.ceil((resetsAt * 1000 - now) / 60_000);
  if (minutes <= 0) return 'reset time has passed';
  const days = Math.floor(minutes / 1440);
  const hours = Math.floor((minutes % 1440) / 60);
  if (days) return `resets in ${days}d ${hours}h`;
  if (hours) return `resets in ${hours}h ${minutes % 60}m`;
  return `resets in ${minutes}m`;
}

export function usageText(name: string, window: UsageWindow, now: number): string {
  const used = `${name} plan usage: ${window.pct}% used`;
  return typeof window.resetsAt === 'number' ? `${used} - ${formatReset(window.resetsAt, now)}` : used;
}
