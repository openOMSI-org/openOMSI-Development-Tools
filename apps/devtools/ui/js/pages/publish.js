// Build and publish: check the plugin, choose whether to sign it, make the .oop.

import { html, useState } from '../../vendor/preact-htm.js';
import { set, useStore, toast } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Notice, Spinner, Tag, Toggle, Select, baseName, fmtSize, copyText } from '../ui.js';
import { call } from '../api.js';
import { runBuild, saveSettings, refreshProblems, openFile } from '../data.js';
import { ensureGame } from './work.js';

export function PublishPage() {
  const s = useStore();
  const p = s.project;
  const errors = s.problems.filter((x) => x.severity === 'error');
  const warnings = s.problems.filter((x) => x.severity === 'warning');
  const keys = s.boot.keys;
  const sign = !!s.settings.sign_builds && keys.length > 0;
  const r = s.lastBuild;
  const [err, setErr] = useState('');

  const build = async () => {
    setErr('');
    try {
      await runBuild(sign);
    } catch (e) {
      setErr(e.message);
    }
  };
  const install = async () => {
    if (!(await ensureGame())) return;
    try {
      const res = await call('run_in_game', { dir: p.dir, launch: false });
      toast(res.mode === 'folder' ? 'Plugin copied into the game.' : 'Installed.', 'ok');
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  const saveAs = async () => {
    const out = await call('pick_save', { title: 'Save the plugin as', name: baseName(r.out) });
    if (!out) return;
    try {
      const res = await call('build', { dir: p.dir, sign, out });
      set({ lastBuild: res });
    } catch (e) {
      toast(e.message, 'error');
    }
  };

  return html`<div>
    <div class="page-head"><div><h1>Build and publish</h1><p>Make the single file other people put into their <code>plugins</code> folder.</p></div></div>
    <div class="stack">
      <${Card} body=${false}>
        <div class=${'step ' + (errors.length ? '' : 'done')}>
          <div class="n">${errors.length ? '1' : html`<${Icon} name="check" size=${18} />`}</div>
          <div class="body">
            <h3>Check</h3>
            <p class="d">${errors.length ? `${errors.length} error(s) must be fixed before the plugin can be built.` : warnings.length ? `No errors, ${warnings.length} warning(s) worth a look.` : 'No problems found.'}</p>
            ${s.problems.slice(0, 5).map((x) => html`<div class=${'problem ' + x.severity} onClick=${() => x.file && openFile(x.file, { line: x.line, col: x.col })}><${Icon} name=${x.severity === 'error' ? 'error' : 'warn'} /><span>${x.message}</span>${x.file && html`<span class="where">${x.file}:${x.line}</span>`}</div>`)}
            <div style="margin-top:8px"><${Button} small icon="refresh" label="Check again" onClick=${refreshProblems} /></div>
          </div>
        </div>
      </${Card}>

      <${Card} body=${false}>
        <div class=${'step ' + (sign ? 'done' : '')}>
          <div class="n">${sign ? html`<${Icon} name="check" size=${18} />` : '2'}</div>
          <div class="body">
            <h3>Sign</h3>
            <p class="d">A signature shows players the plugin really comes from you and was not changed. Unsigned plugins work, but the game warns about them.</p>
            ${keys.length
              ? html`<div class="row wrap" style="margin-top:10px;gap:16px">
                  <${Toggle} checked=${sign} onChange=${(v) => saveSettings({ sign_builds: v })} title="Sign this build" />
                  <div style="width:300px"><${Select} value=${s.settings.default_key} options=${keys.map((k) => [k.path, `${k.label || 'Key'}  (${k.fingerprint})`])} onChange=${(v) => saveSettings({ default_key: v })} /></div>
                </div>`
              : html`<div style="margin-top:10px"><${Button} icon="key" label="Make a signing key" onClick=${() => set({ page: 'keys' })} /></div>`}
          </div>
        </div>
      </${Card}>

      <${Card} body=${false}>
        <div class=${'step ' + (r && !s.busy ? 'done' : '')}>
          <div class="n">${r && !s.busy ? html`<${Icon} name="check" size=${18} />` : '3'}</div>
          <div class="body">
            <h3>Build</h3>
            <p class="d">${p.summary.kind === 'rust' ? 'Compiles your crate to WebAssembly, strips it and packs it.' : 'Compiles your Lua into one protected chunk and packs it. Your source never goes into the file.'}</p>
            <div style="margin-top:10px"><${Button} kind="primary" big icon=${s.busy === 'build' ? undefined : 'package'} disabled=${!!s.busy || errors.length > 0} onClick=${build}>${s.busy === 'build' ? html`<${Spinner} />` : null}Build ${p.summary.id}.oop</${Button}></div>
            ${err && html`<div style="margin-top:12px"><${Notice} kind="danger" icon="error">${err}</${Notice}></div>`}
            ${r && !s.busy && html`<div class="result">
              <${Icon} name="ok" />
              <div class="grow"><b>${baseName(r.out)}</b>  <span class="muted">${fmtSize(r.size)}  ·  ${(r.ms / 1000).toFixed(1)} s</span>
                <div class="row" style="gap:6px;margin-top:2px">${r.signed_by ? html`<${Tag} kind="ok"><${Icon} name="verified" size=${14} />Signed ${r.signed_by}</${Tag}>` : html`<${Tag} kind="warn">Not signed</${Tag}>`}</div>
                <div class="path">${r.out}</div></div>
            </div>
            <div class="row wrap" style="margin-top:12px">
              <${Button} small icon="folder_open" label="Show the file" onClick=${() => call('reveal', { path: r.out })} />
              <${Button} small icon="copy" label="Copy the path" onClick=${async () => (await copyText(r.out)) && toast('Path copied', 'ok', 1500)} />
              <${Button} small icon="download" label="Save a copy as…" onClick=${saveAs} />
              <${Button} small icon="rocket" label="Install into openOMSI" onClick=${install} />
              <${Button} small icon="inspect" label="Inspect" onClick=${() => set({ page: 'inspector', inspectPath: r.out })} />
            </div>`}
          </div>
        </div>
      </${Card}>
    </div>
  </div>`;
}

