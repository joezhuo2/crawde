import { useEffect, useRef, useState, type PointerEvent } from 'react';
import { useWander } from './useWander';
import { command, native } from './native';

export function usePetPosition(interacting: boolean, roaming: boolean, working: boolean, layoutKey: string, onAbility: () => void) {
  const down = useRef<{ x: number; y: number; moved: boolean; ready: Promise<void> } | null>(null);
  const [dragging, setDragging] = useState(false);
  const [keyboardFocused, setKeyboardFocused] = useState(false);
  const wander = useWander(interacting || dragging || keyboardFocused, roaming, working, layoutKey);

  useEffect(() => {
    if (!native) return;
    let frame = 0;
    const send = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const regions = Array.from(document.querySelectorAll('.pet-button, .chat-list.expanded, .chat-bubble, .list-heading, .pagination, .notice, .connect-bubble'))
          .filter(node => getComputedStyle(node).opacity !== '0')
          .map(node => { const r = node.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height }; });
        void command('hit_regions', { regions }).catch(() => {});
      });
    };
    const observer = new MutationObserver(send);
    observer.observe(document.getElementById('root')!, { childList: true, subtree: true, attributes: true, attributeFilter: ['class'] });
    const timer = window.setInterval(send, wander.running ? 40 : 250);
    send();
    return () => { observer.disconnect(); window.clearInterval(timer); cancelAnimationFrame(frame); };
  }, [wander.running]);

  const pointerDown = (event: PointerEvent<HTMLButtonElement>) => {
    if (event.button !== 0) return;
    wander.stop();
    setDragging(true);
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
    if (drag.moved) wander.placed();
    setDragging(false);
    if (!drag.moved && event.type !== 'pointercancel') onAbility();
  };
  return { petRef: wander.ref, running: wander.running, facing: wander.facing, handlers: {
    onPointerDown: pointerDown, onPointerMove: pointerMove, onPointerUp: pointerUp, onPointerCancel: pointerUp,
    onFocus: (event: React.FocusEvent<HTMLButtonElement>) => setKeyboardFocused(event.currentTarget.matches(':focus-visible')),
    onBlur: () => setKeyboardFocused(false),
  } };
}
