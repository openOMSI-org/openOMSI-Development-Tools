// The plugin's details: what the .oop header says about it, as a form that saves itself.

import { html, useState, useEffect, useRef } from '../../vendor/preact-htm.js';
import { S, useStore, toast } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Field, Input, Segmented, Select, Notice, Tag, Toggle } from '../ui.js';
import { call } from '../api.js';
import { saveManifest, openFile } from '../data.js';

const LICENSES = [
  ['MIT', 'MIT (anyone may use and change it)'],
  ['Apache-2.0', 'Apache 2.0'],
  ['GPL-3.0-only', 'GPL 3.0'],
  ['AGPL-3.0-only', 'AGPL 3.0'],
  ['MPL-2.0', 'MPL 2.0'],
  ['CC0-1.0', 'CC0 (public domain)'],
  ['Unlicense', 'Unlicense'],
  ['LicenseRef-Proprietary', 'All rights reserved'],
];

const LEVELS = [
  [0, 'Plain', 'Only formatting is removed. Easiest to debug.'],
  [1, 'Renamed', 'Variable names are replaced as well.'],
  [2, 'Protected', 'Text and numbers are encoded too. The default.'],
];

function bump(v, part) {
  const n = String(v || '0.1.0').split('.').map((x) => parseInt(x, 10) || 0);
  while (n.length < 3) n.push(0);
  if (part === 'major') return `${n[0] + 1}.0.0`;
  if (part === 'minor') return `${n[0]}.${n[1] + 1}.0`;
  return `${n[0]}.${n[1]}.${n[2] + 1}`;
}

export function DetailsPage() {
  const s = useStore();
  const [m, setM] = useState(() => structuredClone(s.project.manifest));
  const [error, setError] = useState('');
  const [saved, setSaved] = useState(true);
  const timer = useRef(0);
  const perms = s.docs?.permissions || [];
  const lua = m.plugin.kind === 'lua';

  // (the manifest was changed from outside: the editor, a permission fix)
  useEffect(() => {
    if (saved) setM(structuredClone(s.project.manifest));
  }, [s.project.manifest]);

  const change = (f) => {
    const next = structuredClone(m);
    f(next);
    setM(next);
    setSaved(false);
    clearTimeout(timer.current);
    timer.current = setTimeout(async () => {
      try {
        await saveManifest(next);
        setError('');
        setSaved(true);
      } catch (e) {
        setError(e.message);
      }
    }, 600);
  };
  const p = m.plugin;
  const luaFiles = [];
  const walk = (nodes) => nodes.forEach((n) => (n.dir ? walk(n.children) : n.name.endsWith('.lua') && !n.path.startsWith('assets/') && luaFiles.push(n.path)));
  walk(s.project.tree);
  const needed = s.needed || {};
  const matchCode = () =>
    change((x) => {
      x.plugin.permissions = Object.keys(needed);
    });

  return html`<div>
    <div class="page-head">
      <div><h1>Details</h1><p>What players and the game see about your plugin. Changes are saved as you type.</p></div>
      <div class="actions"><${Tag} kind=${error ? 'danger' : saved ? 'ok' : ''}>${error ? 'Not saved' : saved ? 'Saved' : 'Saving…'}</${Tag}>
        <${Button} icon="data" label="Edit as text" onClick=${() => openFile('openomsi-plugin.toml')} title="Open openomsi-plugin.toml in the editor" /></div>
    </div>
    ${error && html`<div style="margin-bottom:16px"><${Notice} kind="danger" icon="error">${error}</${Notice}></div>`}
    <div class="stack">
      <${Card} title="About">
        <div class="form-grid">
          <${Field} label="Name" help="Shown in the game's plugin list."><${Input} value=${p.name} onInput=${(v) => change((x) => (x.plugin.name = v))} /></${Field}>
          <${Field} label="Version" help="Raise it for every release.">
            <div class="input-row"><${Input} value=${p.version} onInput=${(v) => change((x) => (x.plugin.version = v))} mono />
              <${Button} small label="+ patch" title="Bug fixes: 0.1.0 to 0.1.1" onClick=${() => change((x) => (x.plugin.version = bump(x.plugin.version, 'patch')))} />
              <${Button} small label="+ minor" title="New features: 0.1.0 to 0.2.0" onClick=${() => change((x) => (x.plugin.version = bump(x.plugin.version, 'minor')))} />
              <${Button} small label="+ major" title="Big changes: 0.1.0 to 1.0.0" onClick=${() => change((x) => (x.plugin.version = bump(x.plugin.version, 'major')))} /></div>
          </${Field}>
          <${Field} label="Description" class="full" help="A sentence or two for the plugin list."><${Input} value=${p.description} onInput=${(v) => change((x) => (x.plugin.description = v))} /></${Field}>
          <${Field} label="Authors" help="Separate several with commas."><${Input} value=${(p.authors || []).join(', ')} onInput=${(v) => change((x) => (x.plugin.authors = v.split(',').map((a) => a.trim()).filter(Boolean)))} /></${Field}>
          <${Field} label="Plugin id" help="Unique and permanent: players' saved data is kept under it. Lower case letters, digits, dots and underscores."><${Input} value=${p.id} onInput=${(v) => change((x) => (x.plugin.id = v))} mono /></${Field}>
        </div>
      </${Card}>

      <${Card} title="Code" hint=${lua ? 'A Lua plugin' : 'A Rust plugin, built to WebAssembly'}>
        <div class="form-grid">
          ${lua
            ? html`<${Field} label="Starts at" help="The script the game runs first. Others are loaded with require()."><${Select} value=${p.entry || 'main.lua'} options=${(luaFiles.length ? luaFiles : ['main.lua']).map((f) => [f, f])} onChange=${(v) => change((x) => (x.plugin.entry = v))} /></${Field}>
               <${Field} label="Protection when building" help=${LEVELS.find((l) => l[0] === m.lua.obfuscation_level)?.[2]}>
                 <${Segmented} full value=${m.lua.obfuscation_level} onChange=${(v) => change((x) => (x.lua.obfuscation_level = v))} options=${LEVELS.map((l) => [l[0], l[1]])} /></${Field}>`
            : html`<${Field} label="Cargo features" help="Extra features passed to cargo build, separated by commas."><${Input} value=${(m.rust.features || []).join(', ')} onInput=${(v) => change((x) => (x.rust.features = v.split(',').map((a) => a.trim()).filter(Boolean)))} mono /></${Field}>`}
          <${Field} label="Needs openOMSI" help=${`The oldest game that can run it. The functions you use came with ${s.boot.api_version} at the latest.`}>
            <div class="input-row"><${Input} value=${p.min_openomsi || ''} onInput=${(v) => change((x) => (x.plugin.min_openomsi = v.trim() || null))} placeholder="any version" mono />
              <${Button} small label=${'Use ' + s.boot.api_version} onClick=${() => change((x) => (x.plugin.min_openomsi = s.boot.api_version))} /></div>
          </${Field}>
        </div>
      </${Card}>

      <${Card} title="Permissions" hint="Tick only what the plugin really needs. The game tells players what a plugin may do before they use it."
        actions=${lua && html`<${Button} small icon="sparkle" label="Match my code" title="Tick exactly the permissions the code uses" onClick=${matchCode} />`}>
        <div class="perm-list">
          ${perms.map((pm) => {
            const on = (p.permissions || []).includes(pm.name);
            const use = needed[pm.name];
            return html`<div class=${'perm' + (on ? ' on' : '')} onClick=${() => change((x) => (x.plugin.permissions = on ? x.plugin.permissions.filter((q) => q !== pm.name) : [...x.plugin.permissions, pm.name]))}>
              <${Toggle} checked=${on} onChange=${() => {}} />
              <div><b>${pm.name}</b><span class="d">${pm.description}</span>
                ${use && !on ? html`<div class="need"><${Tag} kind="warn">Your code calls omsi.${use}</${Tag}></div>` : null}
                ${use && on ? html`<div class="need"><${Tag} kind="ok">Used by omsi.${use}</${Tag}></div>` : null}
                ${!use && on && lua ? html`<div class="need"><${Tag}>Not used by the code</${Tag}></div>` : null}</div>
            </div>`;
          })}
        </div>
      </${Card}>

      <${Card} title="Sharing">
        <div class="form-grid">
          <${Field} label="License"><${Select} value=${p.license || ''} options=${[['', 'None chosen'], ...LICENSES]} onChange=${(v) => change((x) => (x.plugin.license = v || null))} /></${Field}>
          <${Field} label="Homepage" help="A page about the plugin (optional)."><${Input} value=${p.homepage || ''} onInput=${(v) => change((x) => (x.plugin.homepage = v.trim() || null))} placeholder="https://" /></${Field}>
        </div>
      </${Card}>
    </div>
  </div>`;
}
