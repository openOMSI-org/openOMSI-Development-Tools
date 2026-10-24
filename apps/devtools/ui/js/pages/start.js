// The start page: make a plugin, open one, pick up where you left off.

import { html, useState, useEffect } from '../../vendor/preact-htm.js';
import { S, useStore, toast } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Dialog, Field, Input, Segmented, Notice, Empty, openDialog, shortPath } from '../ui.js';
import { call } from '../api.js';
import { openProject, refreshBoot } from '../data.js';

const TEMPLATES = [
  ['hello', 'Hello', 'A message when the drive starts and a speed warning. The quickest way to see something happen.'],
  ['panel', 'Panel', 'A card on the screen with the line, the speed, the next stop and a horn button.'],
  ['blank', 'Blank', 'An empty script: you know what you are doing.'],
];

function NewPluginDialog({ close }) {
  const s = useStore();
  const [name, setName] = useState('');
  const [author, setAuthor] = useState(s.settings.author || '');
  const [kind, setKind] = useState('lua');
  const [template, setTemplate] = useState('hello');
  const [folder, setFolder] = useState(s.settings.projects_folder || (s.boot.documents ? s.boot.documents + (s.boot.os === 'windows' ? '\\openOMSI plugins' : '/openOMSI plugins') : ''));
  const [id, setId] = useState('');
  const [idEdited, setIdEdited] = useState(false);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (idEdited) return;
    call('make_id', { author, name }).then(setId);
  }, [author, name, idEdited]);

  const browse = async () => {
    const p = await call('pick_folder', { title: 'Where should the project be made?', start: folder });
    if (p) setFolder(p);
  };
  const create = async () => {
    setBusy(true);
    setError('');
    try {
      const dir = await call('project_create', { req: { folder, name, id: idEdited ? id : '', author, kind, template: kind === 'rust' ? 'hello' : template } });
      close(dir);
    } catch (e) {
      setError(e.message);
      setBusy(false);
    }
  };
  return html`<${Dialog} title="New plugin" close=${close} wide foot=${html`<${Button} label="Cancel" onClick=${() => close(null)} /><${Button} kind="primary" icon="plus" label="Create plugin" disabled=${!name.trim() || !folder.trim() || busy} onClick=${create} />`}>
    <div class="stack">
      <div class="form-grid">
        <${Field} label="Plugin name"><${Input} value=${name} onInput=${setName} placeholder="Stop Announcer" autofocus onEnter=${() => name.trim() && create()} /></${Field}>
        <${Field} label="Your name" help="Goes into the plugin's details and its id."><${Input} value=${author} onInput=${setAuthor} placeholder="your name or nickname" /></${Field}>
      </div>
      <${Field} label="Language">
        <${Segmented} value=${kind} onChange=${setKind} options=${[['lua', 'Lua  (easiest, no tools needed)'], ['rust', 'Rust  (compiled to WebAssembly)']]} />
        ${kind === 'rust' && !s.boot.cargo.found && html`<div class="status-line warn"><${Icon} name="warn" size=${16} />Rust is not installed on this computer. You can make the project now and set Rust up in Settings later.</div>`}
      </${Field}>
      ${kind === 'lua' && html`<${Field} label="Start from"><div class="template-grid">
        ${TEMPLATES.map(([v, t, d]) => html`<button type="button" class=${'template' + (template === v ? ' on' : '')} onClick=${() => setTemplate(v)}><b>${t}</b><span>${d}</span></button>`)}
      </div></${Field}>`}
      <${Field} label="Where to make it" help=${'A folder named after the plugin is made inside it.'}>
        <div class="input-row"><${Input} value=${folder} onInput=${setFolder} mono /><${Button} icon="folder_open" label="Browse…" onClick=${browse} /></div>
      </${Field}>
      <${Field} label="Plugin id" help="Reverse-domain name, unique among plugins. Made from your name and the plugin's; change it only if you need to.">
        <${Input} value=${id} onInput=${(v) => { setId(v); setIdEdited(true); }} mono />
      </${Field}>
      ${error && html`<${Notice} kind="danger" icon="error">${error}</${Notice}>`}
    </div>
  </${Dialog}>`;
}

export async function newPlugin() {
  const dir = await openDialog(NewPluginDialog);
  if (!dir) return;
  try {
    await openProject(dir);
    toast('Plugin created. Press Run in openOMSI to try it.', 'ok');
  } catch (e) {
    toast(e.message, 'error');
  }
}

export async function chooseProject() {
  const dir = await call('pick_folder', { title: 'Open a plugin project (the folder with openomsi-plugin.toml)', start: S.settings.projects_folder || undefined });
  if (!dir) return;
  try {
    await openProject(dir);
  } catch (e) {
    toast(e.message, 'error');
  }
}

export function StartPage() {
  const s = useStore();
  const recent = s.boot.recent;
  const open = async (dir) => {
    try {
      await openProject(dir);
    } catch (e) {
      toast(e.message, 'error');
      refreshBoot();
    }
  };
  return html`<div>
    <div class="hero">
      <img src="logo.svg" alt="" />
      <div>
        <h1>Make plugins for openOMSI</h1>
        <p>Write a script, run it in the game with one button, then build a single signed file to share. No command line needed.</p>
        <div class="actions">
          <${Button} kind="primary" big icon="plus" label="New plugin" onClick=${newPlugin} />
          <${Button} big icon="folder_open" label="Open project…" onClick=${chooseProject} />
        </div>
      </div>
    </div>

    <div class="section-title">Recent projects</div>
    ${recent.length
      ? html`<div class="project-list">${recent.map((p) => html`<button class=${'project-card' + (p.error ? ' broken' : '')} onClick=${() => open(p.dir)} title=${p.dir}>
          <span class=${'glyph ' + p.kind}>${p.error ? '!' : p.kind === 'rust' ? 'RS' : 'LUA'}</span>
          <span class="body"><b>${p.name}</b><span class="sub">${p.error ? p.error : `${p.version}  ·  ${shortPath(p.dir, 40)}`}</span></span>
          <${Icon} name="right" />
        </button>`)}</div>`
      : html`<${Card}><${Empty} icon="code" title="No projects yet">Make your first plugin: it takes half a minute.</${Empty}></${Card}>`}

    <div class="section-title">How it works</div>
    <div class="grid-3" style="display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:14px">
      ${[
        ['edit', 'Write', 'Edit your script in the built-in editor. It knows every openOMSI function and shows mistakes as you type.'],
        ['play', 'Run', 'One button copies the plugin into the game and starts it. Save again and the running game reloads it.'],
        ['package', 'Share', 'Build one .oop file, signed with your key, that works on Windows, macOS and Linux.'],
      ].map(([i, t, d]) => html`<${Card}><div class="row" style="align-items:flex-start;gap:14px"><${Icon} name=${i} size=${26} /><div><b>${t}</b><div class="muted" style="margin-top:4px">${d}</div></div></div></${Card}>`)}
    </div>
  </div>`;
}
