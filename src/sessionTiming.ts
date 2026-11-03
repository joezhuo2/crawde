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
