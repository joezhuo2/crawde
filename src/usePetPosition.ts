import { useEffect, useRef, useState, type PointerEvent } from 'react';
import { useWander } from './useWander';
import { command, native } from './native';
import type { WalkMode } from './wander';
import type { Snapshot } from './types';

export function usePetPosition(interacting: boolean, roaming: boolean, mode: WalkMode, layoutKey: string, onAbility: () => void) {
  const down = useRef<{ x: number; y: number; moved: boolean; ready: Promise<void> } | null>(null);
  const [dragging, setDragging] = useState(false);
  const [keyboardFocused, setKeyboardFocused] = useState(false);
  const wander = useWander(interacting || dragging || keyboardFocused, roaming, mode, layoutKey);

  useEffect(() => {
    if (!native) return;
    let frame = 0;
    const send = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const regions = Array.from(document.querySelectorAll('.pet-button, .chat-list.expanded, .chat-bubble, .list-heading, .pagination, .notice, .connect-bubble'))
          .filter(node => getComputedStyle(node).opacity !== '0')
          .map(node => {
            const r = node.getBoundingClientRect();
            const clip = node.closest('.chat-stack')?.getBoundingClientRect();
            const x = Math.max(r.x, clip?.left ?? r.x), y = Math.max(r.y, clip?.top ?? r.y);
            return { x, y, width: Math.max(0, Math.min(r.right, clip?.right ?? r.right) - x), height: Math.max(0, Math.min(r.bottom, clip?.bottom ?? r.bottom) - y) };
          }).filter(region => region.width > 0 && region.height > 0);
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
    const rect = event.currentTarget.closest('.pet-anchor')!.getBoundingClientRect();
    await drag.ready;
    if (native) {
      const pet = { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
      await command<Snapshot>('drag_end', { pet }).then(snapshot => {
        window.dispatchEvent(new CustomEvent('pet-positioned', { detail: snapshot }));
      }).catch(() => {});
    }
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
