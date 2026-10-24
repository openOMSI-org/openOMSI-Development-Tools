// Settings: where openOMSI and Rust are, how the app looks, how it updates.

import { html, useState, useEffect } from '../../vendor/preact-htm.js';
import { S, set, useStore, toast } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Field, Input, Segmented, Spinner, Toggle, Tag } from '../ui.js';
import { call } from '../api.js';
import { saveSettings, refreshBoot } from '../data.js';
import { checkForUpdate } from '../updates.js';

function Setting({ title, sub, children }) {
  return html`<div class="setting"><div class="name"><b>${title}</b>${sub && html`<span>${sub}</span>`}</div><div class="ctl">${children}</div></div>`;
}

export function SettingsPage() {
  const s = useStore();
  const st = s.settings;
  const [game, setGame] = useState(st.game_folder);
  const [cargo, setCargo] = useState(st.cargo_path);
  const [busy, setBusy] = useState('');
  const g = s.boot.game;
  const c = s.boot.cargo;

  useEffect(() => {
    const t = setTimeout(async () => {
      if (game === st.game_folder) return;
      await saveSettings({ game_folder: game });
      await refreshBoot();
    }, 500);
    return () => clearTimeout(t);
  }, [game]);
  useEffect(() => {
    const t = setTimeout(async () => {
      if (cargo === st.cargo_path) return;
      await saveSettings({ cargo_path: cargo });
      await refreshBoot();
    }, 500);
    return () => clearTimeout(t);
  }, [cargo]);

  const pickGame = async () => {
    const p = await call('pick_folder', { title: "Choose openOMSI's folder", start: game || undefined });
    if (p) setGame(p);
  };
  const findGame = async () => {
    const f = await call('find_game');
    if (f) setGame(f.folder);
    else toast('openOMSI was not found in the usual places. Use Browse.', 'error');
  };
  const pickCargo = async () => {
    const p = await call('pick_file', { title: 'Choose cargo (in the .cargo/bin folder of your user)', extensions: [] });
    if (p) setCargo(p);
  };
  const addTarget = async () => {
    setBusy('target');
    try {
      const info = await call('install_wasm_target');
      set({ boot: { ...S.boot, cargo: info } });
      toast(info.wasm_target ? 'Ready to build Rust plugins' : 'The target could not be added', info.wasm_target ? 'ok' : 'error');
    } catch (e) {
      toast(e.message, 'error');
    }
    setBusy('');
  };
  const pickFolder = async () => {
    const p = await call('pick_folder', { title: 'Where new plugins are made', start: st.projects_folder || undefined });
    if (p) saveSettings({ projects_folder: p });
  };
  const u = s.update;
  return html`<div>
    <div class="page-head"><div><h1>Settings</h1><p>Everything is saved as you change it.</p></div></div>
    <div class="stack">
      <${Card} title="openOMSI" hint="Run in openOMSI copies the plugin into the game's plugins folder.">
        <${Setting} title="openOMSI's folder" sub="The folder the game is in. On a Mac, the folder that holds openOMSI.app.">
          <div class="input-row"><${Input} value=${game} onInput=${setGame} mono placeholder="not set" /><${Button} icon="folder_open" label="Browse…" onClick=${pickGame} /><${Button} icon="search" title="Look in the usual places" onClick=${findGame} /></div>
          ${g.ok ? html`<div class="status-line ok"><${Icon} name="ok" />Found. Plugins go into ${g.plugins}</div>` : game ? html`<div class="status-line warn"><${Icon} name="warn" />${g.problem}</div>` : html`<div class="status-line"><${Icon} name="info" />Not set yet. You are asked for it the first time you press Run.</div>`}
        </${Setting}>
        <${Setting} title="Keep the game up to date while I type" sub="After every save a Lua plugin is copied into the game, which loads it again within a second."><${Toggle} checked=${st.live_in_game} onChange=${(v) => saveSettings({ live_in_game: v })} /></${Setting}>
      </${Card}>

      <${Card} title="Rust" hint="Only for plugins written in Rust. Lua plugins need nothing.">
        <${Setting} title="cargo" sub="Found by itself in the usual places. Set it only if yours is somewhere else.">
          <div class="input-row"><${Input} value=${cargo} onInput=${setCargo} mono placeholder=${c.found ? c.path : 'automatic'} /><${Button} icon="folder_open" label="Browse…" onClick=${pickCargo} /></div>
          ${c.found ? html`<div class="status-line ok"><${Icon} name="ok" />${c.version}</div>` : html`<div class="status-line warn"><${Icon} name="warn" />Rust was not found. Install it from <a onClick=${() => call('open_url', { url: 'https://rustup.rs' })}>rustup.rs</a>, then press Check again.</div>`}
        </${Setting}>
        <${Setting} title="WebAssembly target" sub="Rust plugins are compiled to WebAssembly, which needs one extra piece.">
          ${c.found ? (c.wasm_target ? html`<div class="status-line ok" style="margin:0"><${Icon} name="ok" />Installed</div>` : html`<${Button} icon="download" disabled=${busy === 'target' || !c.rustup} onClick=${addTarget}>${busy === 'target' ? html`<${Spinner} />` : null}Install wasm32-unknown-unknown</${Button}>`) : html`<span class="muted">Needs Rust first.</span>`}
          <div style="margin-top:8px"><${Button} small icon="refresh" label="Check again" onClick=${refreshBoot} /></div>
        </${Setting}>
      </${Card}>

      <${Card} title="You">
        <${Setting} title="Your name" sub="Put into new plugins as their author."><${Input} value=${st.author} onInput=${(v) => saveSettings({ author: v })} placeholder="your name or nickname" /></${Setting}>
        <${Setting} title="Plugin folder" sub="New plugins are made inside it.">
          <div class="input-row"><${Input} value=${st.projects_folder} onInput=${(v) => saveSettings({ projects_folder: v })} mono placeholder="Documents/openOMSI plugins" /><${Button} icon="folder_open" label="Browse…" onClick=${pickFolder} /></div>
        </${Setting}>
        <${Setting} title="Theme"><${Segmented} value=${st.theme} onChange=${(v) => saveSettings({ theme: v })} options=${[['dark', 'Dark'], ['light', 'Light'], ['system', 'Like the system']]} /></${Setting}>
      </${Card}>

      <${Card} title="Updates">
        <${Setting} title="New versions" sub="Looked for when the app starts.">
          <${Segmented} value=${st.update_mode} onChange=${(v) => saveSettings({ update_mode: v })} options=${[['ask', 'Ask me'], ['auto', 'Install by themselves'], ['off', 'Never look']]} />
        </${Setting}>
        <${Setting} title=${'Version ' + s.boot.version} sub=${s.boot.dev_build ? 'A development build: it looks for updates but never replaces itself.' : 'Updates are checked against their SHA-256 before they replace anything.'}>
          <div class="row"><${Button} icon="refresh" disabled=${u.state === 'checking'} onClick=${() => checkForUpdate(true)}>${u.state === 'checking' ? html`<${Spinner} />` : null}Check now</${Button}>
            ${u.state === 'uptodate' && html`<${Tag} kind="ok">Up to date</${Tag}>`}${u.state === 'available' && html`<${Tag} kind="accent">${u.release.version} is available</${Tag}>`}${u.state === 'failed' && html`<${Tag} kind="danger">${u.error}</${Tag}>`}</div>
        </${Setting}>
      </${Card}>

      <${Card} title="About">
        <p class="dim">openOMSI Development Tools ${s.boot.version}. The plugin compiler, the Rust SDK and these tools are MIT licensed.</p>
        <div class="row wrap" style="margin-top:12px">
          <${Button} small icon="external" label="Source code" onClick=${() => call('open_url', { url: 'https://github.com/openOMSI-org/openOMSI-Development-Tools' })} />
          <${Button} small icon="bug" label="Report a problem" onClick=${() => call('open_url', { url: 'https://github.com/openOMSI-org/openOMSI-Development-Tools/issues/new' })} />
          <${Button} small icon="gamepad" label="openOMSI" onClick=${() => call('open_url', { url: 'https://github.com/openOMSI-org/openOMSI' })} />
        </div>
      </${Card}>
    </div>
  </div>`;
}
