import type { Session } from './types';

export const SLEEP_AFTER_MS = 600_000;

export function shouldSleep(sessions: Session[], claudeOpen: boolean, latestPrompt: number, latestActivity: number, now: number) {
  // A live hook is stronger evidence than an executable-name scan, especially for
  // embedded Desktop sessions and npm/Node CLI installations.
  const recentWork = sessions.some(session => session.status !== 'done' && now - session.updatedAt < SLEEP_AFTER_MS);
  if (recentWork) return false;
  const lastActivity = Math.max(latestPrompt, latestActivity, 0, ...sessions.map(s => s.lastPromptAt));
  return !claudeOpen || !lastActivity || now - lastActivity >= SLEEP_AFTER_MS;
}
