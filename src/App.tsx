import { Bell, Plus, PushPin, Gear } from '@phosphor-icons/react';
import { useCallback, useState } from 'react';
import type { Animation } from './animation';
import { Sprite } from './Sprite';
import { ChatBubbles } from './ChatBubbles';
import { useActivity } from './useActivity';
import { command, native } from './native';
import { usePetPosition } from './usePetPosition';

export function App() {
  const activity = useActivity();
  const [hovered, setHovered] = useState(false);
  const [pinned, setPinned] = useState(false);
  const [opened, setOpened] = useState(false);
  const [animation, setAnimation] = useState<Animation>('idle');
  const [notice, setNotice] = useState('');
  const expanded = hovered || pinned || opened;
  const position = usePetPosition(expanded, true, () => setAnimation('ability'));
  const reset = useCallback(() => setAnimation('idle'), []);
  const act = async (name: string, args?: Record<string, unknown>) => {
    try { await command(name, args); setNotice(''); } catch (error) { setNotice(String(error)); }
  };
  return <main className={`pet-scene ${expanded ? 'is-expanded' : ''} ${activity.dockRight ? 'dock-right' : ''} ${activity.dockTop ? 'dock-top' : ''}`} aria-label="Crawde desktop companion"
    onMouseEnter={() => setHovered(true)} onMouseLeave={() => setHovered(false)}
    onContextMenu={event => { event.preventDefault(); void act('show_context_menu'); }}>
    <ChatBubbles sessions={activity.sessions} expanded={expanded} pinned={pinned} onPin={() => setPinned(p => !p)} onOpen={id => void act('open_chat', { id })} />
    {expanded && (activity.bridgeError || activity.connectionError || notice) && <div className="notice" role="status">{activity.bridgeError || activity.connectionError || notice}</div>}
    {expanded && !activity.hooksInstalled && native && <button className="connect-bubble" onClick={() => void act('configure_hooks', { install: true })}>Connect to Claude Code <span>Enable activity hooks</span></button>}
    <div className="pet-anchor">
      <button className="pet-button" aria-label="Play Crawde’s crab ability. Drag to move." {...position.handlers} onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') setAnimation('ability'); }}>
        <Sprite animation={position.running ? 'run' : animation} facing={position.facing} onComplete={reset} />
      </button>
      <div className="pet-controls" aria-hidden={!expanded}>
        <button className="icon-button" tabIndex={expanded ? 0 : -1} title="New chat in current project" onClick={() => void act('new_chat', { id: activity.sessions[0]?.id ?? null })}><Plus size={18} weight="bold" /></button>
        <button className="icon-button notification-button" tabIndex={expanded ? 0 : -1} title="Show current chats" aria-pressed={opened} onClick={() => setOpened(v => !v)}><Bell size={18} />{activity.sessions.some(s => s.status === 'approval') && <i />}</button>
        {pinned && <PushPin size={12} className="pin-indicator" />}
      </div>
    </div>
    {!native && <aside className="preview-label">Crawde · browser preview <button className="icon-button" title="Preview settings" onClick={() => location.search = '?settings&demo'}><Gear size={16} /></button></aside>}
  </main>;
}
