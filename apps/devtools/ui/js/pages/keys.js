// Signing keys: make, import, choose the one builds are signed with.

import { html } from '../../vendor/preact-htm.js';
import { useStore, toast } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Empty, Notice, Tag, ask, confirmBox, copyText, shortPath } from '../ui.js';
import { call } from '../api.js';
import { refreshBoot, saveSettings } from '../data.js';

export function KeysPage() {
  const s = useStore();
  const keys = s.boot.keys;
  const guard = async (f, ok) => {
    try {
      await f();
      await refreshBoot();
      if (ok) toast(ok, 'ok');
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  const create = async () => {
    const label = await ask({ title: 'New signing key', label: 'A name for the key', value: s.settings.author ? `${s.settings.author}'s key` : 'My openOMSI key', confirm: 'Make the key', help: 'Only you see this name. The fingerprint is what players compare.' });
    if (label) guard(() => call('key_create', { label }), 'Key made');
  };
  const importKey = async () => {
    const path = await call('pick_file', { title: 'Choose a signing key file', extensions: ['oopkey'] });
    if (path) guard(() => call('key_import', { path }), 'Key imported');
  };
  const del = async (k) => {
    const ok = await confirmBox({ title: `Delete "${k.label || k.fingerprint}"?`, text: 'Plugins you signed with it stay valid, but you can never sign with this key again unless you have a copy of its file.', confirm: 'Delete the key', danger: true });
    if (ok) guard(() => call('key_delete', { path: k.path }));
  };
  const exportPublic = async (k) => {
    const out = await call('pick_save', { title: 'Save the public key', name: 'my-openomsi-key.oopkey.pub' });
    if (!out) return;
    guard(async () => call('write_text', { path: out, text: await call('key_public', { path: k.path }) }), 'Public key saved');
  };
  return html`<div>
    <div class="page-head">
      <div><h1>Signing keys</h1><p>A key signs your plugins. Players see "signed by" and your fingerprint in the game, so they know a plugin is yours and was not changed.</p></div>
      <div class="actions"><${Button} icon="download" label="Import…" onClick=${importKey} /><${Button} kind="primary" icon="plus" label="New key" onClick=${create} /></div>
    </div>
    ${keys.length === 0
      ? html`<${Card}><${Empty} icon="key" title="No signing key yet">Make one and every plugin you build can carry your signature.<div style="margin-top:14px"><${Button} kind="primary" icon="plus" label="Make my first key" onClick=${create} /></div></${Empty}></${Card}>`
      : html`<div class="stack">${keys.map((k) => {
          const mine = s.settings.default_key === k.path;
          return html`<div class="key-card">
            <span class="glyph"><${Icon} name="key" /></span>
            <div class="body"><b>${k.label || 'Unnamed key'}</b> ${mine && html`<${Tag} kind="accent">Used for builds</${Tag}>`}
              <div class="fp">${k.fingerprint}</div><div class="muted small ellipsis" title=${k.path}>${shortPath(k.path, 70)}</div></div>
            <div class="row" style="gap:6px">
              ${!mine && html`<${Button} small label="Use for builds" onClick=${() => saveSettings({ default_key: k.path, sign_builds: true })} />`}
              <${Button} small icon="copy" title="Copy the fingerprint" onClick=${async () => (await copyText(k.fingerprint)) && toast('Fingerprint copied', 'ok', 1500)} />
              <${Button} small icon="upload" title="Save the public key (safe to share)" onClick=${() => exportPublic(k)} />
              <${Button} small icon="folder_open" title="Show the key file" onClick=${() => call('reveal', { path: k.path })} />
              <${Button} small kind="danger" icon="trash" title="Delete" onClick=${() => del(k)} />
            </div>
          </div>`;
        })}</div>`}
    <div style="margin-top:22px"><${Notice} icon="shield">Keep the key file private and make a backup copy: anyone who has it can sign as you, and if you lose it you cannot sign updates under the same fingerprint. The public key (the upload button) is safe to share.</${Notice}></div>
  </div>`;
}
