export function SettingsSkeleton({ error }: { error?: string | null }) {
  return <main className="settings-shell settings-loading" aria-busy={!error} aria-label="Loading Crawde settings">
    <header className="settings-header skeleton-header" aria-hidden="true"><div><div className="skeleton skeleton-title" /><div className="skeleton skeleton-subtitle" /></div><div className="skeleton skeleton-avatar" /></header>
    <div className="settings-tabs skeleton-tabs" aria-hidden="true">{[0, 1, 2, 3].map(i => <div key={i} className="skeleton skeleton-tab" />)}</div>
    <div className="settings-content">
      <div className="settings-page" aria-hidden="true"><div className="skeleton skeleton-section" />{[0, 1, 2, 3, 4, 5].map(i => <div className="setting-row skeleton-row" key={i}><div><div className="skeleton skeleton-label" /><div className="skeleton skeleton-detail" /></div><div className="skeleton skeleton-control" /></div>)}</div>
      <footer className="settings-footer"><span role="status">{error ? `Could not load settings: ${error}. Retrying…` : 'Loading your preferences…'}</span><div className="skeleton skeleton-save" aria-hidden="true" /></footer>
    </div>
  </main>;
}
