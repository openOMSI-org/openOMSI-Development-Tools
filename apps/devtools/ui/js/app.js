// The frame: the menu on the left, the page beside it, dialogs and messages above.

import { html, render } from '../vendor/preact-htm.js';
import { useStore, set, S, toast } from './store.js';
import { Icon } from './icons.js';
import { Spinner, Toasts, DialogHost, Notice } from './ui.js';
import { boot, applyTheme, closeProject, saveAll } from './data.js';
import { onDrop, call, inTauri } from './api.js';
import { StartPage, newPlugin, chooseProject } from './pages/start.js';
import { WorkPage } from './pages/work.js';
import { DetailsPage } from './pages/details.js';
import { PublishPage } from './pages/publish.js';
import { KeysPage } from './pages/keys.js';
import { InspectorPage, inspectFile } from './pages/inspector.js';
import { DocsPage } from './pages/docs.js';
import { SettingsPage } from './pages/settings.js';
import { UpdateDialog, startUpdates } from './updates.js';

const NEEDS_PROJECT = new Set(['work', 'details', 'build']);

const GROUPS = [
  ['Plugin', [['start', 'Start', 'home', StartPage], ['work', 'Editor', 'code', WorkPage], ['details', 'Details', 'tune', DetailsPage], ['build', 'Build and publish', 'package', PublishPage]]],
  ['Tools', [['keys', 'Signing keys', 'key', KeysPage], ['inspector', 'Inspector', 'inspect', InspectorPage], ['docs', 'Documentation', 'book', DocsPage]]],
];
const PAGES = [...GROUPS.flatMap(([, p]) => p), ['settings', 'Settings', 'settings', SettingsPage]];

function Sidebar() {
  const s = useStore();
  const u = s.update;
  const go = (id) => {
    if (NEEDS_PROJECT.has(id) && !s.project) {
      toast('Open or make a plugin first.', 'info', 2500);
      return set({ page: 'start' });
    }
    set({ page: id });
  };
  return html`<nav class="sidebar">
    <div class="brand"><img src="logo.svg" alt="" /><div><b>openOMSI</b><small>Dev Tools</small></div></div>
    ${GROUPS.map(([title, pages]) => html`<div class="nav-group" role="group" aria-label=${title}>
      <div class="nav-title">${title}</div>
      ${pages.map(([id, label, icon]) => html`<button class=${'nav-item' + (s.page === id ? ' active' : '') + (NEEDS_PROJECT.has(id) && !s.project ? ' disabled' : '')} onClick=${() => go(id)}>
        <${Icon} name=${icon} /><span class="label">${label}</span>
      </button>
      ${id === 'start' && s.project && html`<div class="nav-sub" title=${s.project.dir}>${s.project.summary.name}</div>`}`)}
    </div>`)}
    <div class="nav-fill"></div>
    <div class="sidebar-foot">
      ${u.state === 'available' && !s.updateDialog && html`<button class="update-chip" onClick=${() => set({ updateDialog: true })}><${Icon} name="update" /><span>Update available<small>Version ${u.release.version}</small></span></button>`}
      <button class=${'nav-item' + (s.page === 'settings' ? ' active' : '')} onClick=${() => go('settings')}><${Icon} name="settings" /><span class="label">Settings</span></button>
      <div class="version-line">Version ${s.boot?.version}</div>
    </div>
  </nav>`;
}

function App() {
  const s = useStore();
  if (!s.ready) {
    return html`<div class="splash">${s.errors?.boot ? html`<div style="max-width:520px"><${Notice} kind="danger" icon="error">${s.errors.boot}</${Notice}></div>` : html`<${Spinner} />`}</div>`;
  }
  const page = PAGES.find((x) => x[0] === s.page && (!NEEDS_PROJECT.has(x[0]) || s.project)) || PAGES[0];
  const Page = page[3];
  return html`<div class="shell">
    <${Sidebar} />
    <main class="main"><div class=${'page page-' + (page[0] === 'work' ? 'work' : page[0])} key=${page[0]}><${Page} /></div></main>
    ${s.dropOver && html`<div class="drop-overlay">Drop a .oop to inspect it, or a project folder to open it</div>`}
    <${DialogHost} />
    <${UpdateDialog} />
    <${Toasts} list=${s.toasts} />
  </div>`;
}

render(html`<${App} />`, document.getElementById('app'));

// keyboard: Cmd/Ctrl+Enter runs the plugin in the game, Cmd/Ctrl+Shift+S saves everything
addEventListener('keydown', async (e) => {
  const mod = e.metaKey || e.ctrlKey;
  if (mod && e.key === 'Enter' && S.project && S.page === 'work') {
    e.preventDefault();
    document.querySelector('.ws-bar .btn.primary')?.click();
  } else if (mod && e.shiftKey && e.key.toLowerCase() === 's' && S.project) {
    e.preventDefault();
    saveAll().then(() => toast('Everything saved', 'ok', 1500));
  }
});

// files dropped on the window: a .oop is inspected, a folder is opened as a project
onDrop(
  async (paths) => {
    const p = paths[0];
    if (!p || !S.ready) return;
    if (/\.oop$/i.test(p)) return inspectFile(p);
    try {
      const { openProject } = await import('./data.js');
      await openProject(p);
    } catch (e) {
      toast(e.message, 'error');
    }
  },
  (over) => over !== S.dropOver && set({ dropOver: over }),
);

matchMedia('(prefers-color-scheme: light)').addEventListener('change', () => applyTheme());
window.addEventListener('beforeunload', () => {});

boot()
  .then(() => (S.boot?.selftest ? import('./selftest.js').then((m) => m.selftest()) : startUpdates()))
  // (in a browser, `#work` and so on show a page with made-up data: the pictures of the docs)
  .then(() => !inTauri && location.hash && import('./mock.js').then(async (m) => m.showcase(location.hash, { S, loadDocs: (await import('./data.js')).loadDocs })))
  .catch((e) => set({ errors: { boot: String(e.message || e) } }));
