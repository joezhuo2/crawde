import { useEffect, useRef, useState, type PointerEvent } from 'react';
import { command, native } from './native';

export function usePetPosition(interacting: boolean, roaming: boolean, onAbility: () => void) {
  const down = useRef<{ x: number; y: number; moved: boolean; ready: Promise<void> } | null>(null);
  const [running, setRunning] = useState(false);
  const [facing, setFacing] = useState(1);
  const runTimer = useRef<number>(0);

  useEffect(() => {
    if (!native) return;
    void command('set_interacting', { interacting }).catch(() => {});
    if (interacting) setRunning(false);
  }, [interacting]);

  useEffect(() => {
    if (!native || interacting || !roaming) return;
    const timer = window.setInterval(async () => {
      try {
        const delta = await command<number>('roam', { direction: Math.random() > .5 ? 1 : -1 });
        if (delta) {
          setFacing(delta > 0 ? 1 : -1); setRunning(true);
          runTimer.current = window.setTimeout(() => setRunning(false), 1250);
        }
      } catch { /* Native diagnostics remain visible through the activity snapshot. */ }
    }, 35_000 + Math.random() * 25_000);
    return () => { window.clearInterval(timer); window.clearTimeout(runTimer.current); setRunning(false); };
  }, [interacting, roaming]);

  useEffect(() => {
    if (!native) return;
    let frame = 0;
    const send = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const regions = Array.from(document.querySelectorAll('.pet-anchor, .chat-bubble, .list-heading, .pagination, .notice, .connect-bubble'))
          .filter(node => getComputedStyle(node).opacity !== '0')
          .map(node => { const r = node.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; });
        void command('hit_regions', { regions }).catch(() => {});
      });
    };
    const observer = new MutationObserver(send);
    observer.observe(document.getElementById('root')!, { childList: true, subtree: true, attributes: true });
    const timer = window.setInterval(send, 250);
    send();
    return () => { observer.disconnect(); window.clearInterval(timer); cancelAnimationFrame(frame); };
  }, []);

  const pointerDown = (event: PointerEvent<HTMLButtonElement>) => {
    if (event.button !== 0) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    down.current = { x: event.screenX, y: event.screenY, moved: false,
      ready: native ? command('drag_start').catch(() => {}) : Promise.resolve() };
  };
  const pointerMove = async (event: PointerEvent<HTMLButtonElement>) => {
    const drag = down.current;
    if (!drag) return;
    if (Math.hypot(event.screenX - drag.x, event.screenY - drag.y) > 5) drag.moved = true;
    if (drag.moved && native) { await drag.ready; await command('drag_move').catch(() => {}); }
  };
  const pointerUp = async (event: PointerEvent<HTMLButtonElement>) => {
    const drag = down.current;
    down.current = null;
    if (!drag) return;
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
    await drag.ready;
    if (native) await command('drag_end').catch(() => {});
    if (!drag.moved && event.type !== 'pointercancel') onAbility();
  };
  return { running, facing, handlers: { onPointerDown: pointerDown, onPointerMove: pointerMove, onPointerUp: pointerUp, onPointerCancel: pointerUp } };
}
