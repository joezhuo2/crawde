import { expect, it } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';
import { SettingsSkeleton } from './SettingsSkeleton';

it('offers a loading status without editable default preferences', () => {
  const html = renderToStaticMarkup(<SettingsSkeleton />);
  expect(html).toContain('aria-busy="true"');
  expect(html).toContain('Loading your preferences');
  expect(html).not.toContain('<input');
});

it('exposes connection errors instead of silently loading forever', () => {
  expect(renderToStaticMarkup(<SettingsSkeleton error="Connection timed out" />)).toContain('Connection timed out');
});
