import type { Session } from './types';

export function formatDuration(ms: number): string {
  const total = Math.floor(Math.max(0, ms) / 1000);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  if (hours) return `${hours}h ${minutes}m`;
  if (minutes) return `${minutes}m ${seconds}s`;
  return `${seconds}s`;
}

export function sessionTiming(session: Session, now: number): string {
  const running = Math.max(0, now - session.statusSince);
  const working = session.workingMs + (session.status === 'working' ? running : 0);
  const waiting = session.waitingMs + (session.status === 'approval' ? running : 0);
  const parts = [`${formatDuration(working)} working`];
  if (waiting >= 1000) parts.push(`${formatDuration(waiting)} waiting`);
  parts.push(`active ${formatDuration(now - session.updatedAt)} ago`);
  return parts.join(' · ');
}

export function toolSummary(session: Session): { text: string; detail: string } {
  const tools = session.toolCount ?? 0;
  const files = session.filesEdited?.length ?? 0;
  const parts: string[] = [];
  if (tools) parts.push(`${tools} ${tools === 1 ? 'tool' : 'tools'}`);
  if (files) parts.push(`${files} ${files === 1 ? 'file' : 'files'}`);
  const breakdown = Object.entries(session.toolCounts ?? {}).sort((a, b) => b[1] - a[1]).map(([name, count]) => `${name} ×${count}`);
  const detail = [breakdown.join(', '), ...(session.filesEdited ?? [])].filter(Boolean).join('\n');
  return { text: parts.join(' · '), detail };
}
