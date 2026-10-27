import { useEffect, useState } from 'react';
import { command, native } from './native';
import type { Session, Snapshot } from './types';

const initial: Snapshot = { sessions: [], hooksInstalled: false, bridgeError: null };

// Preview data is opt-in and never substituted for a disconnected native bridge.
function demo(): Session[] {
  return [
    { id: 'preview-1', project: 'Crawde', title: 'A little companion for your work', status: 'working', activity: '📚 Reading the animation frames', tool: 'Read', agents: ['illustrator', 'explorer'] },
    { id: 'preview-2', project: 'Crawde', title: 'Polish the settings', status: 'approval', activity: '✋ Waiting for your approval', tool: 'Bash', agents: [] },
    { id: 'preview-3', project: 'Garden', title: 'Tidy up the stylesheet', status: 'done', activity: '✓ Finished', tool: '', agents: [] },
    { id: 'preview-4', project: 'Garden', title: 'Review the next change', status: 'working', activity: '✏️ Editing the layout', tool: 'Edit', agents: [] },
  ].map(s => ({ ...s, cwd: 'C:\\Projects\\' + s.project, updatedAt: Date.now(), lastPromptAt: Date.now(), finishedAt: null }) as Session);
}

export function useActivity() {
  const [snapshot, setSnapshot] = useState<Snapshot>(() => !native && new URLSearchParams(location.search).has('demo')
    ? { ...initial, sessions: demo(), hooksInstalled: true, claudeOpen: true } : initial);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    if (!native) return;
    let live = true;
    let timeout: number;
    let layoutRevision = 0;
    const positioned = (event: Event) => {
      layoutRevision++;
      setSnapshot((event as CustomEvent<Snapshot>).detail);
    };
    window.addEventListener('pet-positioned', positioned);
    async function poll() {
      const revision = layoutRevision;
      try {
        const next = await command<Snapshot>('snapshot');
        if (live && revision === layoutRevision) { setSnapshot(next); setError(null); }
      } catch (e) { if (live) setError(String(e)); }
      if (live) timeout = window.setTimeout(poll, 700);
    }
    void poll();
    return () => { window.removeEventListener('pet-positioned', positioned); live = false; window.clearTimeout(timeout); };
  }, []);
  return { ...snapshot, connectionError: error };
}
