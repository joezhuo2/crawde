import { useCallback, useEffect, useState } from 'react';
import type { Animation } from './animation';
import type { Session } from './types';
import { shouldSleep } from './sleep';

export function useBehavior(sessions: Session[], claudeOpen: boolean, reduceMotion: boolean, latestPrompt = 0, latestActivity = 0) {
  const [clock, setClock] = useState(Date.now());
  const [action, setAction] = useState<Animation | null>(null);
  const [systemReduced, setSystemReduced] = useState(() => matchMedia('(prefers-reduced-motion: reduce)').matches);
  const reduced = reduceMotion || systemReduced;
  const sleeping = shouldSleep(sessions, claudeOpen, latestPrompt, latestActivity, clock);
  const approval = sessions.some(s => s.status === 'approval');
  const complete = useCallback(() => setAction(null), []);
  const ability = useCallback(() => setAction('ability'), []);

  useEffect(() => {
    const timer = window.setInterval(() => setClock(Date.now()), 2000);
    const media = matchMedia('(prefers-reduced-motion: reduce)');
    const change = () => setSystemReduced(media.matches);
    media.addEventListener('change', change);
    return () => { clearInterval(timer); media.removeEventListener('change', change); };
  }, []);

  useEffect(() => {
    if (!approval || reduced) return;
    const strike = () => setAction(current => current === 'ability' ? current : Math.random() > .5 ? 'attackA' : 'attackB');
    strike();
    const timer = window.setInterval(strike, 22_000);
    return () => clearInterval(timer);
  }, [approval, reduced]);

  return { sleeping, reduced, animation: action ?? (sleeping ? 'sleep' : 'idle'), complete, ability };
}
