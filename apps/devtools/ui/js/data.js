// What the pages share: starting up, settings, the open project, its files in the editor, and
// the build and run actions.

import { S, set, toast, logLine } from './store.js';
import { call, on } from './api.js';

// --- start and settings ----------------------------------------------------------------

export function applyTheme() {
  const t = S.settings?.theme || 'dark';
  const light = t === 'light' || (t === 'system' && matchMedia('(prefers-color-scheme: light)').matches);
  document.documentElement.dataset.theme = light ? 'light' : 'dark';
}

export async function boot() {
  const b = await call('boot');
  set({ boot: b, settings: b.settings, ready: true });
  applyTheme();
  on('build-log', (e) => logLine(e.level, e.text));
  loadDocs().catch(() => {});
  return b;
}

/** The documentation pages and the API manifest (packed into the program), loaded once. */
export async function loadDocs() {
  if (S.docs) return S.docs;
  const docs = await call('docs');
  set({ docs });
  return docs;
}

export async function saveSettings(patch) {
  const s = await call('save_settings', { patch });
  set({ settings: s });
  applyTheme();
  return s;
}

export async function refreshBoot() {
  const b = await call('boot');
  set({ boot: b, settings: b.settings });
}

// --- the project -----------------------------------------------------------------------

export async function openProject(dir) {
  const r = await call('project_open', { dir });
  set({ project: { dir, ...r }, tabs: [], active: null, problems: [], log: [], needed: {}, live: false, page: 'work' });
  await refreshBoot();
  refreshProblems();
  // the entry file is the first thing to see
  const entry = r.summary.kind === 'rust' ? 'src/lib.rs' : r.summary.entry;
  if (entry) openFile(entry).catch(() => {});
  return r;
}

export function closeProject() {
  set({ project: null, tabs: [], active: null, problems: [], page: 'start', live: false });
}

export async function refreshTree() {
  if (!S.project) return;
  const tree = await call('project_tree', { dir: S.project.dir });
  set({ project: { ...S.project, tree } });
}

export async function reloadManifest() {
  if (!S.project) return;
  const r = await call('project_open', { dir: S.project.dir });
  set({ project: { ...S.project, summary: r.summary, manifest: r.manifest, tree: r.tree } });
}

/** Writes the manifest the details page edited; the error text when it is not valid. */
export async function saveManifest(manifest) {
  await call('manifest_save', { dir: S.project.dir, manifest });
  set({ project: { ...S.project, manifest } });
  // (the editor may have the manifest's text open)
  const tab = S.tabs.find((t) => t.path === 'openomsi-plugin.toml');
  if (tab) {
    const f = await call('file_read', { dir: S.project.dir, path: tab.path });
    set({ tabs: S.tabs.map((t) => (t === tab ? { ...t, text: f.text, saved: f.text, rev: (t.rev || 0) + 1 } : t)) });
  }
  const summary = await call('project_open', { dir: S.project.dir }).then((r) => r.summary);
  set({ project: { ...S.project, summary } });
  refreshProblems();
}

let problemTimer = 0;
export function refreshProblems() {
  clearTimeout(problemTimer);
  problemTimer = setTimeout(async () => {
    if (!S.project) return;
    try {
      const [problems, needed] = await Promise.all([call('check_project', { dir: S.project.dir }), S.project.summary.kind === 'lua' ? call('needed_permissions', { dir: S.project.dir }) : {}]);
      set({ problems, needed });
    } catch {
      return;
    }
  }, 250);
}

// --- files in the editor ---------------------------------------------------------------

export async function openFile(path, at) {
  const have = S.tabs.find((t) => t.path === path);
  if (have) {
    set({ active: path, page: 'work', goto: at ? { path, ...at, n: Date.now() } : null });
    return;
  }
  const f = await call('file_read', { dir: S.project.dir, path });
  set({ tabs: [...S.tabs, { path, text: f.text, saved: f.text, binary: f.binary, size: f.size, rev: 0 }], active: path, page: 'work', goto: at ? { path, ...at, n: Date.now() } : null });
}

export function closeTab(path) {
  const i = S.tabs.findIndex((t) => t.path === path);
  if (i < 0) return;
  const tabs = S.tabs.filter((t) => t.path !== path);
  let active = S.active;
  if (active === path) active = tabs[Math.min(i, tabs.length - 1)]?.path || null;
  set({ tabs, active });
}

const saveTimers = new Map();

export function setTabText(path, text) {
  set({ tabs: S.tabs.map((t) => (t.path === path ? { ...t, text } : t)) });
  clearTimeout(saveTimers.get(path));
  saveTimers.set(path, setTimeout(() => saveTab(path, true).catch((e) => toast(e.message, 'error')), 900));
}

let afterTimer = 0;
export async function saveTab(path, auto) {
  const tab = S.tabs.find((t) => t.path === path);
  if (!tab || tab.binary || tab.text === tab.saved) return;
  clearTimeout(saveTimers.get(path));
  const text = tab.text;
  await call('file_write', { dir: S.project.dir, path, text });
  set({ tabs: S.tabs.map((t) => (t.path === path ? { ...t, saved: text } : t)) });
  if (path === 'openomsi-plugin.toml') reloadManifest().catch(() => {});
  clearTimeout(afterTimer);
  afterTimer = setTimeout(afterSave, 350);
  if (!auto) toast('Saved ' + path, 'ok', 1500);
}

export async function saveAll() {
  for (const t of S.tabs) if (t.text !== t.saved) await saveTab(t.path, true);
}

async function afterSave() {
  refreshProblems();
  if (S.live && S.project?.summary.kind === 'lua') {
    try {
      const r = await call('run_in_game', { dir: S.project.dir, launch: false });
      if (r.changed) set({ liveStamp: new Date() });
    } catch (e) {
      logLine('error', e.message);
    }
  }
}

export const isDirty = (t) => t.text !== t.saved;

// --- build and run ---------------------------------------------------------------------

export async function runBuild(sign) {
  await saveAll();
  set({ busy: 'build', dock: 'output', dockOpen: true, log: [] });
  try {
    const r = await call('build', { dir: S.project.dir, sign });
    set({ lastBuild: r });
    return r;
  } finally {
    set({ busy: '' });
    refreshProblems();
  }
}

/** Puts the plugin in the game (and starts it). Resolves false when the game's folder is not set. */
export async function runInGame(launch) {
  await saveAll();
  set({ busy: 'run', dock: 'output', dockOpen: true, log: [] });
  try {
    const r = await call('run_in_game', { dir: S.project.dir, launch });
    if (r.mode === 'folder' && S.settings.live_in_game) set({ live: true, liveStamp: new Date() });
    return r;
  } catch (e) {
    if (e.message.startsWith('needs-game:')) return false;
    logLine('error', e.message);
    throw e;
  } finally {
    set({ busy: '' });
    refreshProblems();
  }
}
