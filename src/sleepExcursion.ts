// Each cycle starts only after a full sleeping interval. Cleanup cancels every
// pending action when real activity, reduced motion, or preferences intervene.
export function scheduleSleepExcursions(wake: (awake: boolean) => void, attack: () => void, random = Math.random) {
  const timers = new Set<ReturnType<typeof setTimeout>>();
  const later = (callback: () => void, delay: number) => {
    const timer = setTimeout(() => { timers.delete(timer); callback(); }, delay);
    timers.add(timer);
  };
  const schedule = () => later(() => {
    const duration = 15_000 + random() * 30_000;
    const count = 1 + Math.floor(random() * 3);
    wake(true);
    for (let i = 0; i < count; i++) later(attack, duration * (i + 1) / (count + 1));
    later(() => { wake(false); schedule(); }, duration);
  }, 60_000 + random() * 180_000);
  schedule();
  return () => { timers.forEach(clearTimeout); timers.clear(); };
}
