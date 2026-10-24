// The inspector: what is inside a .oop, who signed it, and a way to sign one.

import { html, useState, useEffect } from '../../vendor/preact-htm.js';
import { S, set, useStore, toast } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Notice, Select, Tag, fmtSize, baseName } from '../ui.js';
import { call } from '../api.js';

export async function inspectFile(path) {
  try {
    const r = await call('inspect', { path });
    set({ inspected: r, page: 'inspector', inspectPath: null });
  } catch (e) {
    toast(e.message, 'error');
  }
}

export function InspectorPage() {
  const s = useStore();
  const r = s.inspected;
  const [key, setKey] = useState('');
  useEffect(() => {
    if (s.inspectPath) inspectFile(s.inspectPath);
  }, [s.inspectPath]);
  useEffect(() => {
    if (!key && s.boot.keys.length) setKey(s.settings.default_key || s.boot.keys[0].path);
  }, [s.boot.keys]);
  const pick = async () => {
    const p = await call('pick_file', { title: 'Choose a .oop plugin', extensions: ['oop'] });
    if (p) inspectFile(p);
  };
  const sign = async () => {
    try {
      const fp = await call('sign_file', { file: r.path, key });
      toast('Signed by ' + fp, 'ok');
      inspectFile(r.path);
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  const h = r?.header || {};
  const row = (k, v) => v ? html`<dt>${k}</dt><dd>${v}</dd>` : null;
  return html`<div>
    <div class="page-head"><div><h1>Inspector</h1><p>Open a .oop file to see its details, permissions and signature. Its code is never shown.</p></div>
      <div class="actions"><${Button} kind="primary" icon="folder_open" label="Open a .oop…" onClick=${pick} /></div></div>
    ${!r
      ? html`<div class=${'dropzone' + (s.dropOver ? ' over' : '')}><${Icon} name="package" size=${36} /><div>Drop a .oop file on this window, or press Open.</div></div>`
      : html`<div class="stack">
        <${Card}>
          <div class="row" style="gap:12px;margin-bottom:14px"><${Icon} name="package" size=${28} /><div class="grow"><h2 style="font-size:18px">${h.name} <span class="muted">${h.version}</span></h2><div class="muted small selectable">${r.path}</div></div>
            ${r.intact ? html`<${Tag} kind="ok">File intact</${Tag}>` : html`<${Tag} kind="danger">Cannot be read</${Tag}>`}</div>
          <dl class="kv">
            ${row('Id', html`<code>${h.id}</code>`)}
            ${row('Description', h.description)}
            ${row('Kind', `${h.kind}  ·  starts at ${h.entry}`)}
            ${row('Authors', (h.authors || []).join(', '))}
            ${row('License', h.license)}
            ${row('Needs openOMSI', h.min_openomsi)}
            ${row('Permissions', (h.permissions || []).length ? html`<div class="pill-row">${h.permissions.map((p) => html`<${Tag}>${p}</${Tag}>`)}</div>` : 'none: it only reads the game')}
            ${row('Built', [h.tool, h.created].filter(Boolean).join('  ·  '))}
            ${row('Size', fmtSize(r.size))}
          </dl>
        </${Card}>
        <${Card} title="Signature">
          ${r.signed
            ? r.valid
              ? html`<div class="row"><${Tag} kind="ok"><${Icon} name="verified" size=${14} />Valid</${Tag}><span>Signed by <code class="selectable">${r.signer}</code>. It was not changed since.</span></div>`
              : html`<${Notice} kind="danger" icon="error">The file carries a signature that does not match: it was changed after signing. Do not trust it.</${Notice}>`
            : html`<div class="row wrap" style="gap:14px"><${Tag} kind="warn">Not signed</${Tag}>
                ${s.boot.keys.length
                  ? html`<div style="width:280px"><${Select} value=${key} options=${s.boot.keys.map((k) => [k.path, `${k.label || 'Key'}  (${k.fingerprint})`])} onChange=${setKey} /></div><${Button} icon="verified" label="Sign this file" onClick=${sign} />`
                  : html`<span class="muted">Make a signing key first (Signing keys).</span>`}</div>`}
        </${Card}>
        <${Card} title=${`Files (${r.files.length})`} body=${false}>
          <table class="table"><tbody>${r.files.map((f) => html`<tr><td class="mono">${f.path}</td><td class="muted" style="text-align:right">${fmtSize(f.size)}</td></tr>`)}</tbody></table>
          ${!r.files.length && html`<div class="muted" style="padding:14px 20px">The file list could not be read.</div>`}
        </${Card}>
      </div>`}
  </div>`;
}
