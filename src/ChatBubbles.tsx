import { ArrowLeft, ArrowRight, Check, PushPin, PushPinSlash, X } from '@phosphor-icons/react';
import { useEffect, useRef, useState } from 'react';
import type { Session } from './types';

interface Props {
  sessions: Session[];
  expanded: boolean;
  pinned: boolean;
  onPin: () => void;
  onOpen: (id: string) => void;
  onDismiss: (id: string) => void;
}

export function ChatBubbles({ sessions, expanded, pinned, onPin, onOpen, onDismiss }: Props) {
  const [page, setPage] = useState(0);
  const [phase, setPhase] = useState<'steady' | 'out' | 'in'>('steady');
  const pageTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pages = Math.max(1, Math.ceil(sessions.length / 3));
  const pageCount = useRef(pages);
  pageCount.current = pages;
  useEffect(() => { setPage(p => Math.min(p, pages - 1)); }, [pages]);
  useEffect(() => () => { if (pageTimer.current) clearTimeout(pageTimer.current); }, []);
  useEffect(() => {
    if (phase !== 'in') return;
    let secondFrame = 0;
    const firstFrame = requestAnimationFrame(() => { secondFrame = requestAnimationFrame(() => setPhase('steady')); });
    return () => { cancelAnimationFrame(firstFrame); cancelAnimationFrame(secondFrame); };
  }, [phase]);
  const goToPage = (next: number) => {
    if (phase !== 'steady' || next < 0 || next >= pages || next === page) return;
    const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches || document.documentElement.dataset.reduced === 'true';
    if (reduced) { setPage(next); return; }
    setPhase('out');
    pageTimer.current = setTimeout(() => { setPage(Math.min(next, pageCount.current - 1)); setPhase('in'); }, 200);
  };
  const visible = sessions.slice(page * 3, page * 3 + 3);
  if (!sessions.length) return null;

  return <section className={`chat-list ${expanded ? 'expanded' : 'collapsed'}`} style={{ '--count': visible.length } as React.CSSProperties} aria-label="Active Claude chats">
    <div className="list-heading">
      <span>{sessions.length} {sessions.length === 1 ? 'conversation' : 'conversations'}</span>
      <button className="icon-button pin" tabIndex={expanded ? 0 : -1} title={pinned ? 'Unpin chats' : 'Keep chats expanded'} aria-pressed={pinned} onClick={onPin}>
        {pinned ? <PushPinSlash size={16} /> : <PushPin size={16} />}
      </button>
    </div>
    <div className={`bubbles page-${phase}`} aria-busy={phase !== 'steady'}>
      {visible.map((session, index) => <article key={session.id} className={`chat-bubble ${session.status}`} style={{ '--index': index } as React.CSSProperties}>
        <button className="chat-open" onClick={() => onOpen(session.id)} aria-label={`Open ${session.project} · ${session.title}`}>
        <span className="chat-title"><span>{session.project} <span className="separator">·</span> {session.title}</span>{session.status === 'done' && <Check weight="bold" size={15} />}</span>
        <span className="chat-details">
          <span className="chat-activity">{session.activity}</span>
          <span className="chat-meta">{session.agents.length
            ? `🤝 ${session.agents.length} subagent${session.agents.length === 1 ? '' : 's'} running`
            : session.tool ? `🛠️ ${session.tool}` : session.status === 'done' ? '🌿 All wrapped up' : '💭 Working on your request'}</span>
        </span>
        </button>
        <button className="icon-button chat-close" aria-label={`Dismiss ${session.title}`} onClick={() => onDismiss(session.id)}><X size={12} weight="bold" /></button>
      </article>)}
    </div>
    {pages > 1 && <nav className="pagination" aria-label="More chats">
      <button className="icon-button" disabled={page === 0 || phase !== 'steady'} onClick={() => goToPage(page - 1)} aria-label="Previous chats"><ArrowLeft size={14} /></button>
      <span>{page + 1} / {pages} <span className="muted">· {sessions.length} chats</span></span>
      <button className="icon-button" disabled={page === pages - 1 || phase !== 'steady'} onClick={() => goToPage(page + 1)} aria-label="More chats"><ArrowRight size={14} /></button>
    </nav>}
  </section>;
}
