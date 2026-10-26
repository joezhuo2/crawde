import { ArrowLeft, ArrowRight, Check, PushPin, PushPinSlash } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import type { Session } from './types';

interface Props {
  sessions: Session[];
  expanded: boolean;
  pinned: boolean;
  onPin: () => void;
  onOpen: (id: string) => void;
}

export function ChatBubbles({ sessions, expanded, pinned, onPin, onOpen }: Props) {
  const [page, setPage] = useState(0);
  const pages = Math.max(1, Math.ceil(sessions.length / 3));
  useEffect(() => { setPage(p => Math.min(p, pages - 1)); }, [pages]);
  const visible = sessions.slice(page * 3, page * 3 + 3);
  if (!sessions.length) return null;

  return <section className={`chat-list ${expanded ? 'expanded' : 'collapsed'}`} aria-label="Active Claude chats">
    <div className="list-heading">
      <span>{sessions.length} {sessions.length === 1 ? 'conversation' : 'conversations'}</span>
      <button className="icon-button pin" title={pinned ? 'Unpin chats' : 'Keep chats expanded'} aria-pressed={pinned} onClick={onPin}>
        {pinned ? <PushPinSlash size={16} /> : <PushPin size={16} />}
      </button>
    </div>
    <div className="bubbles">
      {visible.map((session, index) => <button key={session.id} className={`chat-bubble ${session.status}`} style={{ '--index': index } as React.CSSProperties}
        onClick={() => onOpen(session.id)} title={`Open ${session.project} · ${session.title}`}>
        <span className="chat-title"><span>{session.project} <span className="separator">·</span> {session.title}</span>{session.status === 'done' && <Check weight="bold" size={15} />}</span>
        <span className="chat-details">
          <span className="chat-activity">{session.activity}</span>
          <span className="chat-meta">{session.agents.length
            ? `🤝 ${session.agents.length} subagent${session.agents.length === 1 ? '' : 's'} running`
            : session.tool ? `🛠️ ${session.tool}` : session.status === 'done' ? '🌿 All wrapped up' : '💭 Working on your request'}</span>
        </span>
      </button>)}
    </div>
    {pages > 1 && <nav className="pagination" aria-label="More chats">
      <button className="icon-button" disabled={page === 0} onClick={() => setPage(p => p - 1)} aria-label="Previous chats"><ArrowLeft size={14} /></button>
      <span>{page + 1} / {pages} <span className="muted">· {sessions.length} chats</span></span>
      <button className="icon-button" disabled={page === pages - 1} onClick={() => setPage(p => p + 1)} aria-label="More chats"><ArrowRight size={14} /></button>
    </nav>}
  </section>;
}
