import { ArrowSquareOut, Check, X, SpeakerHigh, SlidersHorizontal, PlugsConnected, Info } from '@phosphor-icons/react';
import { useEffect, useState, type ReactNode } from 'react';
import { defaults, type Preferences } from './preferences';
import { useActivity } from './useActivity';
import { command, native } from './native';
import { Sprite } from './Sprite';
import { SettingsSkeleton } from './SettingsSkeleton';
import { version } from '../package.json';

function Toggle({ label, detail, checked, disabled, onChange }: { label: string; detail?: string; checked: boolean; disabled?: boolean; onChange: (v: boolean) => void }) {
  return <label className="setting-row"><span>{label}{detail && <small>{detail}</small>}</span><input className="toggle" type="checkbox" checked={checked} disabled={disabled} onChange={e => onChange(e.target.checked)} /></label>;
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return <label className="setting-row"><span>{label}</span>{children}</label>;
}

export function Settings() {
  const activity = useActivity();
  const [draft, setDraft] = useState<Preferences | null>(native ? null : defaults);
  const [tab, setTab] = useState('companion');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const preferences = draft ?? defaults;
  useEffect(() => { if (activity.preferences && !draft) setDraft(activity.preferences); }, [activity.preferences, draft]);
  useEffect(() => {
    const enabled = activity.preferences?.enabled;
    if (enabled !== undefined) setDraft(current => current ? { ...current, enabled } : current);
  }, [activity.preferences?.enabled]);
  useEffect(() => { document.documentElement.dataset.theme = preferences.theme; document.documentElement.dataset.reduced = String(preferences.reducedMotion); }, [preferences.theme, preferences.reducedMotion]);
  const set = <K extends keyof Preferences>(key: K, value: Preferences[K]) => { setDraft(p => ({ ...(p ?? defaults), [key]: value })); setMessage(''); };
  const act = async (name: string, args?: Record<string, unknown>, success?: string) => {
    setBusy(true);
    try { await command(name, args); setMessage(success ?? ''); } catch (error) { setMessage(String(error)); }
    finally { setBusy(false); }
  };

  const setVisibility = async (visible: boolean) => {
    setBusy(true);
    try { await command('set_visibility', { visible }); set('enabled', visible); }
    catch (error) { setMessage(String(error)); }
    finally { setBusy(false); }
  };

  if (!draft || new URLSearchParams(location.search).has('loading')) return <SettingsSkeleton error={activity.connectionError} />;

  return <main className="settings-shell">
    <header className="settings-header"><div><h1>Crawde</h1><p>A little company while Claude works.</p></div><Sprite animation="idle" scale={2} reducedMotion={preferences.reducedMotion} /></header>
    <nav className="settings-tabs" aria-label="Settings categories">{[
      ['companion', 'Companion', SlidersHorizontal], ['sounds', 'Sounds', SpeakerHigh], ['claude', 'Claude', PlugsConnected], ['about', 'About', Info],
    ].map(([id, label, Icon]) => { const TabIcon = Icon as typeof Info; return <button key={id as string} aria-current={tab === id ? 'page' : undefined} onClick={() => setTab(id as string)}><TabIcon size={17} />{label as string}</button>; })}</nav>
    <form className="settings-content" onSubmit={e => { e.preventDefault(); void act('save_preferences', { preferences }, 'Preferences saved.'); }}>
      <div className="settings-page" key={tab}>
        {tab === 'companion' && <>
          {!native && <p className="settings-note" role="status">Browser preview. Desktop visibility and saving are available in the Crawde Windows app.</p>}
          <h2>Your companion</h2>
          <Toggle label="Show Crawde" detail="Tray click or hotkey toggles visibility." checked={preferences.enabled} disabled={busy || !native} onChange={v => void setVisibility(v)} />
          <Toggle label="Launch with Windows" detail="Wait in the tray until Claude opens." checked={preferences.startup} onChange={v => set('startup', v)} />
          <Field label="Show / hide shortcut"><input className="hotkey-input" aria-label="Global hotkey" value={preferences.hotkey} onChange={e => set('hotkey', e.target.value)} placeholder="Ctrl+Alt+P" spellCheck={false} /></Field>
          <Field label="Appearance"><select value={preferences.theme} onChange={e => set('theme', e.target.value as Preferences['theme'])}><option value="system">Follow system</option><option value="light">Light</option><option value="dark">Dark</option></select></Field>
          <Field label="Pet size"><select value={preferences.scale} onChange={e => set('scale', Number(e.target.value))}><option value={2}>Small · 2× pixels</option><option value={3}>Medium · 3× pixels</option><option value={4}>Large · 4× pixels</option></select></Field>
          <h2>Movement</h2>
          <Toggle label="Little walks" detail="Occasional short runs near its saved spot." checked={preferences.roaming} onChange={v => set('roaming', v)} />
          <Toggle label="Reduce motion" detail="Still sprite; no roaming or repeated attacks." checked={preferences.reducedMotion} onChange={v => set('reducedMotion', v)} />
          <p className="settings-note">Drag Crawde to move it. Positions are remembered per monitor. It sleeps after 10 minutes without a prompt, or when Claude closes.</p>
        </>}
        {tab === 'sounds' && <>
          <h2>A gentle heads-up</h2>
          <Toggle label="Sound effects" checked={preferences.sound} onChange={v => set('sound', v)} />
          <Field label="Volume"><input aria-label="Sound volume" type="range" min="0" max="1" step="0.05" value={preferences.volume} onChange={e => set('volume', Number(e.target.value))} /></Field>
          <Field label="Sound style"><select value={preferences.soundStyle} onChange={e => set('soundStyle', e.target.value as Preferences['soundStyle'])}><option value="soft">Soft tones</option><option value="retro">Retro game</option></select></Field>
          <Toggle label="Needs approval" checked={preferences.approvalSound} onChange={v => set('approvalSound', v)} />
          <Toggle label="Finished a response" checked={preferences.completionSound} onChange={v => set('completionSound', v)} />
          <Toggle label="Errors" checked={preferences.errorSound} onChange={v => set('errorSound', v)} />
          <Toggle label="Approval reminders" detail="A quiet reminder every 45 seconds." checked={preferences.reminders} onChange={v => set('reminders', v)} />
          <button type="button" className="secondary-button" onClick={() => void act('test_sound', { preferences })}>Test sound</button>
        </>}
        {tab === 'claude' && <>
          <h2>Connected to your work</h2>
          <div className={`connection-status ${activity.hooksInstalled ? 'connected' : ''}`}><Check size={17} />{activity.hooksInstalled ? 'Claude Code hooks installed' : 'Activity hooks are not connected'}</div>
          {activity.hooksInstalled && <p className="settings-note" role="status">{activity.lastEventAt ? `Activity received ${Math.max(0, Math.floor((Date.now() - activity.lastEventAt) / 1000))} seconds ago.` : 'No activity received yet. Start a new Claude Code session after connecting hooks.'}</p>}
          <p className="settings-note">Supports Claude Code in your terminal and the Code tab in Claude Desktop. Crawde observes activity and leaves every approval to you.</p>
          <button type="button" className="secondary-button" disabled={busy} onClick={() => void act('configure_hooks', { install: !activity.hooksInstalled }, activity.hooksInstalled ? 'Disconnected. Existing hooks were preserved.' : 'Connected. Restart existing Claude sessions to load hooks.')}>
            {activity.hooksInstalled ? 'Disconnect activity hooks' : 'Connect activity hooks'}
          </button>
          <Field label="Open chats in"><select value={preferences.openTarget} onChange={e => set('openTarget', e.target.value as Preferences['openTarget'])}><option value="desktop">Claude Desktop</option><option value="terminal">Windows Terminal</option></select></Field>
          <p className="settings-note">New chat uses the most recently active project. A clicked conversation becomes the current project.</p>
          {activity.bridgeError && <p className="settings-error">{activity.bridgeError}</p>}
        </>}
        {tab === 'about' && <>
          <h2>Small crab. Big company.</h2>
          <p className="settings-note">Crawde {version} · Made for Claude Code on Windows.<br />Your activity stays on this computer.</p>
          <Toggle label="Check for updates on open" detail="Checks GitHub releases. Install from the right-click menu." checked={preferences.checkOnOpen} onChange={v => set('checkOnOpen', v)} />
          {activity.update && <p className="settings-note" role="status">{activity.update.message}</p>}
          <button type="button" className="text-button" onClick={() => void act('open_releases')}>View releases <ArrowSquareOut size={14} /></button>
          <div className="quit-section"><button type="button" className="text-button" onClick={() => void act('quit')}>Quit Crawde</button><small>Stops the tray app until its next launch.</small></div>
        </>}
      </div>
      <footer className="settings-footer"><span role="status">{message || activity.connectionError}</span><button className="primary-button" disabled={busy || !draft || !native} type="submit">{busy ? 'Saving…' : 'Save changes'}</button></footer>
    </form>
    {!native && <button className="preview-close icon-button" title="Back to pet preview" onClick={() => location.search = '?demo'}><X size={16} /></button>}
  </main>;
}
