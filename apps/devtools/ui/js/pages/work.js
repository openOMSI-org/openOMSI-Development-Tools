// The workspace: the project's files, the editor, and the panel with problems, output and the
// game's log. The buttons on top put the plugin in the game and build it.

import { html, useState, useEffect, useRef } from '../../vendor/preact-htm.js';
import { S, set, useStore, toast, logLine } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Dialog, Field, Input, Notice, Spinner, Tag, ask, confirmBox, openDialog, baseName, fmtSize } from '../ui.js';
import { call } from '../api.js';
import { EditorHost, languageOf } from '../editor.js';
import { openFile, closeTab, setTabText, saveTab, saveAll, isDirty, refreshTree, refreshProblems, runBuild, runInGame, saveSettings, saveManifest } from '../data.js';

// --- where is the game? ----------------------------------------------------------------

function GameFolderDialog({ close }) {
  const s = useStore();
  const [path, setPath] = useState(s.settings.game_folder || '');
  const [info, setInfo] = useState(null);
  useEffect(() => {
    if (!path.trim()) return setInfo(null);
    const t = setTimeout(() => call('game_info', { path }).then(setInfo), 200);
    return () => clearTimeout(t);
  }, [path]);
  const browse = async () => {
    const p = await call('pick_folder', { title: 'Choose openOMSI\'s folder (the one with the game and its plugins folder)', start: path || undefined });
    if (p) setPath(p);
  };
  const detect = async () => {
    const g = await call('find_game');
    if (g) setPath(g.folder);
    else toast('openOMSI was not found in the usual places. Choose its folder with Browse.', 'error');
  };
  return html`<${Dialog} title="Where is openOMSI?" close=${close} foot=${html`<${Button} label="Cancel" onClick=${() => close(null)} /><${Button} kind="primary" label="Use this folder" disabled=${!info?.ok} onClick=${() => close(info.folder)} />`}>
    <div class="stack">
      <p class="dim">The plugin is copied into the <code>plugins</code> folder of the game, which then loads it. Choose the folder where openOMSI is installed (on a Mac, the folder that holds openOMSI.app).</p>
      <${Field} label="openOMSI's folder"><div class="input-row"><${Input} value=${path} onInput=${setPath} mono /><${Button} icon="folder_open" label="Browse…" onClick=${browse} /><${Button} icon="search" label="Find it" onClick=${detect} /></div></${Field}>
      ${info && (info.ok
        ? html`<div class="status-line ok"><${Icon} name="ok" />openOMSI found. Plugins go into ${info.plugins}</div>`
        : html`<div class="status-line warn"><${Icon} name="warn" />${info.problem}</div>`)}
    </div>
  </${Dialog}>`;
}

/** The game's folder is set, or the person is asked for it. False when they did not. */
export async function ensureGame() {
  if (S.boot.game?.ok) return true;
  const folder = await openDialog(GameFolderDialog);
  if (!folder) return false;
  await saveSettings({ game_folder: folder });
  const g = await call('game_info', { path: folder });
  set({ boot: { ...S.boot, game: g } });
  return true;
}

// --- the file tree ---------------------------------------------------------------------

function worstMark(path, dir, problems) {
  let worst = '';
  for (const p of problems) {
    const hit = dir ? p.file.startsWith(path + '/') : p.file === path;
    if (!hit) continue;
    if (p.severity === 'error') return 'error';
    worst = 'warning';
  }
  return worst;
}

const iconFor = (name) => {
  const ext = name.split('.').pop().toLowerCase();
  if (ext === 'lua') return 'code';
  if (ext === 'rs') return 'code';
  if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'].includes(ext)) return 'image';
  if (['wav', 'ogg', 'mp3', 'flac'].includes(ext)) return 'music';
  if (['toml', 'json'].includes(ext)) return 'data';
  return 'file';
};

function TreeRows({ nodes, depth, expanded, toggle, act }) {
  const s = S;
  return nodes.map((n) => {
    const open = expanded.has(n.path);
    const mark = worstMark(n.path, n.dir, s.problems);
    return html`<div key=${n.path}>
      <div class=${'tree-row' + (s.active === n.path ? ' on' : '')} style=${`padding-left:${6 + depth * 14}px`}
        onClick=${() => (n.dir ? toggle(n.path) : openFile(n.path).catch((e) => toast(e.message, 'error')))} title=${n.path}>
        <${Icon} name=${n.dir ? (open ? 'folder_open' : 'folder') : iconFor(n.name)} size=${16} />
        <span class="name">${n.name}</span>
        ${mark && html`<span class=${'mark ' + mark}></span>`}
        <span class="acts" onClick=${(e) => e.stopPropagation()}>
          ${n.dir && html`<button title="New file here" onClick=${() => act('file', n)}><${Icon} name="file_add" /></button><button title="New folder here" onClick=${() => act('folder', n)}><${Icon} name="folder_add" /></button>`}
          ${n.path !== 'openomsi-plugin.toml' && html`<button title="Rename" onClick=${() => act('rename', n)}><${Icon} name="edit" /></button><button title="Delete" onClick=${() => act('delete', n)}><${Icon} name="trash" /></button>`}
        </span>
      </div>
      ${n.dir && open && html`<${TreeRows} nodes=${n.children} depth=${depth + 1} expanded=${expanded} toggle=${toggle} act=${act} />`}
    </div>`;
  });
}

function Tree() {
  const s = useStore();
  const [expanded, setExpanded] = useState(() => new Set(['assets']));
  const toggle = (p) =>
    setExpanded((e) => {
      const n = new Set(e);
      n.has(p) ? n.delete(p) : n.add(p);
      return n;
    });
  const dir = s.project.dir;
  const guard = async (f) => {
    try {
      await f();
      await refreshTree();
      refreshProblems();
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  const act = async (what, node) => {
    const base = node ? (node.dir ? node.path + '/' : node.path.includes('/') ? node.path.replace(/[^/]*$/, '') : '') : '';
    if (node?.dir) setExpanded((e) => new Set(e).add(node.path));
    if (what === 'file' || what === 'folder') {
      const name = await ask({ title: what === 'file' ? 'New file' : 'New folder', label: what === 'file' ? 'File name (a folder/ prefix makes folders)' : 'Folder name', confirm: 'Create', value: what === 'file' ? 'util.lua' : '' });
      if (!name) return;
      await guard(async () => {
        await call('file_create', { dir, path: base + name, folder: what === 'folder' });
        if (what === 'file') await openFile(base + name);
      });
    } else if (what === 'rename') {
      const name = await ask({ title: 'Rename', label: 'New path', value: node.path, confirm: 'Rename' });
      if (!name || name === node.path) return;
      await guard(async () => {
        await call('file_rename', { dir, from: node.path, to: name });
        // (open tabs follow the file)
        set({ tabs: S.tabs.map((t) => (t.path === node.path ? { ...t, path: name } : t.path.startsWith(node.path + '/') ? { ...t, path: name + t.path.slice(node.path.length) } : t)), active: S.active === node.path ? name : S.active });
      });
    } else if (what === 'delete') {
      const ok = await confirmBox({ title: 'Delete ' + node.name + '?', text: node.dir ? 'The folder and everything in it is deleted for good.' : 'The file is deleted for good.', confirm: 'Delete', danger: true });
      if (!ok) return;
      await guard(async () => {
        await call('file_delete', { dir, path: node.path });
        for (const t of [...S.tabs]) if (t.path === node.path || t.path.startsWith(node.path + '/')) closeTab(t.path);
      });
    } else if (what === 'import') {
      const from = await call('pick_file', { title: 'Choose a file to add to the plugin (a picture, a sound, a script)', extensions: [] });
      if (!from) return;
      await guard(() => call('file_import', { dir, from, to_dir: node?.dir ? node.path : 'assets' }));
    }
  };
  return html`<aside class="tree">
    <div class="tree-head"><span>FILES</span><span class="row" style="gap:0">
      <${Button} kind="ghost" small icon="file_add" title="New file" onClick=${() => act('file', null)} />
      <${Button} kind="ghost" small icon="folder_add" title="New folder" onClick=${() => act('folder', null)} />
      <${Button} kind="ghost" small icon="upload" title="Add a picture, a sound or another file to assets" onClick=${() => act('import', null)} />
    </span></div>
    <div class="tree-list"><${TreeRows} nodes=${s.project.tree} depth=${0} expanded=${expanded} toggle=${toggle} act=${act} /></div>
  </aside>`;
}

// --- tabs and the editor ---------------------------------------------------------------

function Tabs() {
  const s = useStore();
  return html`<div class="ftabs">${s.tabs.map((t) => html`<div class=${'ftab' + (s.active === t.path ? ' on' : '')} onClick=${() => set({ active: t.path })} title=${t.path}>
    <span>${baseName(t.path)}</span>
    ${isDirty(t) ? html`<span class="dirty"></span>` : null}
    <button class="x" title="Close" onClick=${(e) => { e.stopPropagation(); closeTab(t.path); }}><${Icon} name="x" /></button>
  </div>`)}</div>`;
}

function Editors({ onCursor }) {
  const s = useStore();
  const api = s.docs?.api;
  if (!api) return html`<div class="editor-empty"><${Spinner} /></div>`;
  const active = s.tabs.find((t) => t.path === s.active);
  return html`<div class="editor-wrap">
    ${s.tabs.map((t) => (t.binary
      ? active?.path === t.path && html`<div class="editor-empty" key=${t.path}><${Icon} name="file" /><b>${baseName(t.path)}</b><span>${fmtSize(t.size)}. This kind of file is not edited here.</span>
          <${Button} small icon="folder_open" label="Show in the file manager" onClick=${() => call('reveal', { path: s.project.dir + '/' + t.path })} /></div>`
      : html`<${EditorHost} key=${t.path} dir=${s.project.dir} tab=${t} active=${s.active === t.path} api=${api}
          extra=${s.problems.filter((p) => p.file === t.path && p.severity === 'warning')} goto=${s.goto}
          onText=${(text) => setTabText(t.path, text)} onSave=${() => saveTab(t.path).catch((e) => toast(e.message, 'error'))} onCursor=${onCursor} />`))}
    ${!s.tabs.length && html`<div class="editor-empty"><${Icon} name="code" /><b>Pick a file on the left</b><span>${s.project.summary.kind === 'rust' ? 'src/lib.rs is where your plugin starts.' : s.project.summary.entry + ' is where your plugin starts.'}</span></div>`}
  </div>`;
}

// --- the dock --------------------------------------------------------------------------

function ProblemsList() {
  const s = useStore();
  if (!s.problems.length) return html`<div class="log-line good" style="padding-top:8px">No problems found.</div>`;
  const fix = async (perm) => {
    const m = structuredClone(S.project.manifest);
    m.plugin.permissions = [...new Set([...m.plugin.permissions, perm])];
    try {
      await saveManifest(m);
      toast(`Permission "${perm}" added`, 'ok');
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  return s.problems.map((p) => html`<div class=${'problem ' + p.severity} onClick=${() => p.file && openFile(p.file, { line: p.line, col: p.col }).catch(() => {})}>
    <${Icon} name=${p.severity === 'error' ? 'error' : 'warn'} />
    <span>${p.message}</span>
    ${p.fix_permission && html`<${Button} small class="fix" label=${'Add "' + p.fix_permission + '"'} onClick=${(e) => { e.stopPropagation(); fix(p.fix_permission); }} />`}
    ${p.file && html`<span class="where">${p.file}:${p.line}</span>`}
  </div>`);
}

function Output() {
  const s = useStore();
  const end = useRef();
  useEffect(() => end.current?.scrollIntoView({ block: 'end' }), [s.log.length]);
  if (!s.log.length) return html`<div class="log-line dim" style="padding-top:8px">Nothing yet. Press Run in openOMSI or Build.</div>`;
  return html`${s.log.map((l) => html`<div class=${'log-line ' + l.level}>${l.text}</div>`)}<div ref=${end}></div>`;
}

function GameLog() {
  const [lines, setLines] = useState([]);
  const [filter, setFilter] = useState('');
  const end = useRef();
  useEffect(() => {
    let alive = true;
    const tick = () => call('game_log', { lines: 400 }).then((l) => alive && setLines(l)).catch(() => {});
    tick();
    const t = setInterval(tick, 1500);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);
  const shown = filter ? lines.filter((l) => l.toLowerCase().includes(filter.toLowerCase())) : lines;
  useEffect(() => end.current?.scrollIntoView({ block: 'end' }), [shown.length]);
  return html`<div style="position:sticky;top:0;background:var(--surface);padding:6px 0;font-family:Inter,system-ui,sans-serif">
      <input class="input" style="height:28px;max-width:320px" placeholder="Filter the game's log (try: plugin)" value=${filter} onInput=${(e) => setFilter(e.currentTarget.value)} /></div>
    ${shown.length ? shown.map((l) => html`<div class=${'log-line ' + (/error|panic|failed/i.test(l) ? 'error' : /warn/i.test(l) ? 'dim' : 'info')}>${l}</div>`) : html`<div class="log-line dim">The game has not written a log yet. Start openOMSI and drive.</div>`}<div ref=${end}></div>`;
}

function Dock() {
  const s = useStore();
  const errors = s.problems.filter((p) => p.severity === 'error').length;
  const warnings = s.problems.length - errors;
  const tab = (id, label, count, kind) => html`<button class=${'dt' + (s.dock === id && s.dockOpen ? ' on' : '')} onClick=${() => set({ dock: id, dockOpen: s.dock === id ? !s.dockOpen : true })}>${label}${count ? html`<span class=${'count ' + kind}>${count}</span>` : null}</button>`;
  return html`<div class=${'dock' + (s.dockOpen ? '' : ' closed')}>
    <div class="dock-tabs">
      ${tab('problems', 'Problems', errors || warnings, errors ? 'error' : 'warning')}
      ${tab('output', 'Output', 0)}
      ${tab('gamelog', "Game log", 0)}
      <span class="grow" style="flex:1"></span>
      <${Button} kind="ghost" small icon=${s.dockOpen ? 'down' : 'up'} title=${s.dockOpen ? 'Hide the panel' : 'Show the panel'} onClick=${() => set({ dockOpen: !s.dockOpen })} />
    </div>
    ${s.dockOpen && html`<div class="dock-body">${s.dock === 'problems' ? html`<${ProblemsList} />` : s.dock === 'output' ? html`<${Output} />` : html`<${GameLog} />`}</div>`}
  </div>`;
}

// --- the page --------------------------------------------------------------------------

export function WorkPage() {
  const s = useStore();
  const [cursor, setCursor] = useState(null);
  const p = s.project;
  const lua = p.summary.kind === 'lua';
  const active = s.tabs.find((t) => t.path === s.active);

  const run = async (launch) => {
    if (!(await ensureGame())) return;
    try {
      const r = await runInGame(launch);
      if (r === false) return;
      toast(r.mode === 'folder' ? (launch ? 'Plugin installed. openOMSI is starting.' : 'Plugin updated in the game.') : 'Plugin built and installed.', 'ok');
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  const build = async () => {
    try {
      const r = await runBuild(s.settings.sign_builds);
      toast(`Built ${baseName(r.out)}`, 'ok');
    } catch (e) {
      toast(e.message, 'error');
    }
  };
  const toggleLive = async () => {
    if (!s.live && !(await ensureGame())) return;
    set({ live: !s.live });
    if (!s.live === true) run(false);
  };
  const dirty = s.tabs.some(isDirty);
  return html`<div class="ws">
    <div class="ws-bar">
      <div class="title"><b>${p.summary.name}</b><span>${p.summary.id}  ·  ${p.summary.version}  ·  ${lua ? 'Lua' : 'Rust'}</span></div>
      ${lua && html`<button class=${'tag ' + (s.live ? 'ok' : '')} style="cursor:pointer;height:30px;padding:0 12px" onClick=${toggleLive}
        title="While this is on, every save copies the plugin into the running game, which loads it again within a second">
        <${Icon} name="bolt" size=${15} />${s.live ? 'Live in the game' : 'Live is off'}</button>`}
      <${Button} icon="package" label="Build .oop" disabled=${!!s.busy} onClick=${build} title="Build the file you share" />
      <${Button} kind="primary" icon=${s.busy === 'run' ? undefined : 'play'} disabled=${!!s.busy || s.problems.some((x) => x.severity === 'error')} onClick=${() => run(true)}
        title=${s.problems.some((x) => x.severity === 'error') ? 'Fix the errors first' : 'Copy the plugin into openOMSI and start the game'}>
        ${s.busy === 'run' ? html`<${Spinner} />` : null}Run in openOMSI</${Button}>
    </div>
    <div class="ws-body">
      <${Tree} />
      <div class="editor-col">
        <${Tabs} />
        <${Editors} onCursor=${setCursor} />
        <div class="statusbar">
          <span>${active ? active.path : p.dir}</span>
          <span class="grow"></span>
          ${cursor && active && !active.binary && html`<span>Ln ${cursor.line}, Col ${cursor.col}</span>`}
          ${active && !active.binary && html`<span>${{ lua: 'Lua', rust: 'Rust', toml: 'TOML', json: 'JSON', text: 'Text' }[languageOf(active.path)]}</span>`}
          <span class=${dirty ? '' : 'ok'}>${dirty ? 'Saving…' : 'All changes saved'}</span>
          ${s.live && s.liveStamp && html`<span class="ok">Synced ${s.liveStamp.toLocaleTimeString()}</span>`}
        </div>
        <${Dock} />
      </div>
    </div>
  </div>`;
}
