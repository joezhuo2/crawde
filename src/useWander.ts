import { useEffect, useRef, useState } from 'react';
import { nextWalk, walkDelay, type WalkMode } from './wander';

export function useWander(paused: boolean, enabled: boolean, mode: WalkMode, layoutKey: string) {
  const ref = useRef<HTMLButtonElement>(null);
  const offset = useRef(0);
  const home = useRef(0);
  const motion = useRef<Animation | null>(null);
  const [running, setRunning] = useState(false);
  const [facing, setFacing] = useState(1);

  const stop = () => {
    const element = ref.current;
    if (element && motion.current) {
      offset.current = new DOMMatrixReadOnly(getComputedStyle(element).transform).m41;
      element.style.transform = `translateX(${offset.current}px)`;
      motion.current.cancel();
      motion.current = null;
    }
    setRunning(false);
  };

  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    // Clamp after a dock/size change, including when walks are disabled.
    const rect = element.getBoundingClientRect();
    const min = 6 - (rect.left - offset.current);
    const max = Math.max(min, window.innerWidth - 6 - (rect.right - offset.current));
    offset.current = Math.max(min, Math.min(max, offset.current));
    home.current = Math.max(min, Math.min(max, home.current));
    element.style.transform = `translateX(${offset.current}px)`;
    if (paused || !enabled) return;

    let timer = 0;
    let disposed = false;
    const schedule = () => { timer = window.setTimeout(walk, walkDelay(mode)); };
    const walk = () => {
      if (disposed || document.hidden || element.matches(':focus-visible')) { if (!disposed) schedule(); return; }
      const target = nextWalk(offset.current, home.current, min, max);
      if (Math.abs(target - offset.current) < 1) { schedule(); return; }
      setFacing(target > offset.current ? 1 : -1);
      setRunning(true);
      const animation = element.animate([
        { transform: `translateX(${offset.current}px)` },
        { transform: `translateX(${target}px)` },
      ], { duration: 1_200, easing: 'linear', fill: 'forwards' });
      motion.current = animation;
      animation.onfinish = () => {
        if (disposed) return;
        offset.current = target;
        element.style.transform = `translateX(${target}px)`;
        animation.cancel();
        motion.current = null;
        setRunning(false);
        schedule();
      };
    };
    schedule();
    return () => { disposed = true; window.clearTimeout(timer); stop(); };
  }, [paused, enabled, mode, layoutKey]);

  return { ref, running, facing, stop, placed: () => { home.current = offset.current; } };
}
