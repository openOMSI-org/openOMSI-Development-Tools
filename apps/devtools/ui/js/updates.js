// Updates of the app itself: looked for at start, offered or installed by themselves (the
// setting), checked against their SHA-256 by the program before anything is replaced.

import { html, useEffect } from '../vendor/preact-htm.js';
import { S, set, useStore } from './store.js';
import { Button, Dialog, Tag, Spinner } from './ui.js';
import { call, on } from './api.js';
import { renderMarkdown } from './md.js';
import { saveSettings } from './data.js';

const upd = (patch) => set({ update: { ...S.update, ...patch } });

export async function checkForUpdate(manual) {
  if (S.update.state === 'checking' || S.update.state === 'downloading') return;
  upd({ state: 'checking', error: '' });
  try {
    const release = await call('update_check');
    if (!release) {
      upd({ state: manual ? 'uptodate' : 'idle', release: null });
      return;
    }
    upd({ state: 'available', release });
    const mode = S.settings.update_mode;
    if (manual) set({ updateDialog: true });
    else if (mode === 'auto' && !S.boot.dev_build) installUpdate();
    else if (mode === 'ask' && S.settings.skipped_version !== release.version) set({ updateDialog: true });
  } catch (e) {
    upd({ state: manual ? 'failed' : 'idle', error: e.message });
  }
}

export async function installUpdate() {
  const release = S.update.release;
  upd({ state: 'downloading', done: 0, total: release.size || 0 });
  try {
    await call('update_install', { release });
    upd({ state: 'installed' });
    await call('update_relaunch');
  } catch (e) {
    upd({ state: 'failed', error: e.message });
    set({ updateDialog: true });
  }
}

export function startUpdates() {
  on('update-progress', (p) => upd({ done: p.done, total: p.total }));
  if (S.boot.updates_disabled || S.settings.update_mode === 'off') return;
  // (a moment after the start: the first picture comes first)
  setTimeout(() => checkForUpdate(false), 2500);
}

export function UpdateDialog() {
  const s = useStore();
  const u = s.update;
  if (!s.updateDialog || !u.release && u.state !== 'failed') return null;
  const r = u.release;
  const close = () => set({ updateDialog: false });
  const busy = u.state === 'downloading' || u.state === 'installed';
  return html`<div class="overlay"><${Dialog} title=${u.state === 'failed' ? 'The update did not work' : 'A new version is available'} close=${close}
    foot=${busy ? null : u.state === 'failed'
      ? html`<${Button} label="Close" onClick=${close} /><${Button} kind="primary" label="Download by hand" onClick=${() => call('open_url', { url: r?.page || 'https://github.com/openOMSI-org/openOMSI-Development-Tools/releases/latest' })} />`
      : html`<${Button} label="Skip this version" onClick=${async () => { await saveSettings({ skipped_version: r.version }); close(); }} /><${Button} label="Not now" onClick=${close} /><${Button} kind="primary" icon="download" label="Install and restart" onClick=${installUpdate} />`}>
    ${u.state === 'failed' ? html`<p class="dim selectable">${u.error}</p>` : html`
      <div class="row" style="gap:10px;margin-bottom:12px"><b style="font-size:16px">openOMSI Dev Tools ${r.version}</b><${Tag}>you have ${s.boot.version}</${Tag}></div>
      ${busy
        ? html`<div class="stack"><div class="row"><${Spinner} /><span>${u.state === 'installed' ? 'Installed. Restarting…' : 'Downloading…'}</span></div>
            <div class="progress"><div style=${`width:${u.total ? Math.min(100, (u.done / u.total) * 100) : 5}%`}></div></div></div>`
        : r.notes ? html`<div class="md" style="max-height:300px;overflow:auto" dangerouslySetInnerHTML=${{ __html: renderMarkdown(r.notes) }}></div>` : html`<p class="dim">The app restarts after the update. Your projects and keys stay as they are.</p>`}`}
  </${Dialog}></div>`;
}
