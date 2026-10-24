// The documentation: the guide pages and a searchable reference of every plugin function.

import { html, useState, useMemo } from '../../vendor/preact-htm.js';
import { useStore } from '../store.js';
import { Icon } from '../icons.js';
import { Button, Card, Tag, Spinner, copyText } from '../ui.js';
import { call } from '../api.js';
import { renderMarkdown } from '../md.js';
import { toast } from '../store.js';

const GROUP_TITLES = { events: 'Timers and handlers', util: 'Helpers', vehicle: 'The player\'s vehicle', game: 'The game', map: 'The map', duty: 'Duty and timetable', storage: 'Storage and files', ui: 'Panels and notifications', traffic: 'AI traffic', plugin: 'Plugins', time: 'Time', camera: 'Camera', weather: 'Weather', audio: 'Audio', input: 'Input', people: 'People', lan: 'LAN game', network: 'Network' };

const clean = (s) => s.replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' })[c]).replace(/`([^`]+)`/g, '<code>$1</code>');

function ApiFn({ f }) {
  const params = (f.params || []).map((p) => (p.optional ? `[${p.name}]` : p.name)).join(', ');
  const sig = `omsi.${f.name}(${params})`;
  return html`<div class="api-fn" id=${'fn-' + f.name}>
    <div class="sig">${sig}${f.returns ? html`<span class="ret">  →  ${f.returns}</span>` : null}</div>
    <p dangerouslySetInnerHTML=${{ __html: clean(f.doc || '') }}></p>
    <div class="tags">
      ${f.permission ? html`<${Tag} kind="accent">needs ${f.permission}</${Tag}>` : html`<${Tag}>no permission</${Tag}>`}
      <${Tag}>since ${f.since}</${Tag}>
      <button class="tag" style="cursor:pointer" onClick=${async () => (await copyText(sig)) && toast('Copied', 'ok', 1200)}><${Icon} name="copy" size=${13} />Copy</button>
    </div>
  </div>`;
}

function Reference({ api }) {
  const [q, setQ] = useState('');
  const groups = useMemo(() => {
    const needle = q.trim().toLowerCase();
    const by = new Map();
    for (const f of api.functions) {
      if (needle && !(f.name.toLowerCase().includes(needle) || (f.doc || '').toLowerCase().includes(needle))) continue;
      if (!by.has(f.group)) by.set(f.group, []);
      by.get(f.group).push(f);
    }
    return [...by.entries()];
  }, [q, api]);
  const events = api.events.filter((e) => !q.trim() || e.name.toLowerCase().includes(q.trim().toLowerCase()) || (e.doc || '').toLowerCase().includes(q.trim().toLowerCase()));
  return html`<div class="md" style="max-width:900px">
    <h1>Plugin reference</h1>
    <p>${api.functions.length} functions and ${api.events.length} events of openOMSI ${api.version}. In Lua every function is <code>omsi.${'<name>'}</code>.</p>
    <div class="search" style="margin:14px 0 4px"><input class="input" placeholder="Search the functions and events" value=${q} onInput=${(e) => setQ(e.currentTarget.value)} /></div>
    ${groups.map(([g, list]) => html`<div class="api-group"><h2>${GROUP_TITLES[g] || g}</h2>${list.map((f) => html`<${ApiFn} f=${f} />`)}</div>`)}
    ${events.length > 0 && html`<div class="api-group"><h2>Event names (for omsi.on)</h2>${events.map((e) => html`<div class="api-fn"><div class="sig">omsi.on("${e.name}", function(${e.args.join(', ')}) ... end)</div><p dangerouslySetInnerHTML=${{ __html: clean(e.doc || '') }}></p><div class="tags"><${Tag}>since ${e.since}</${Tag}></div></div>`)}</div>`}
    ${!groups.length && !events.length && html`<p>Nothing matches "${q}".</p>`}
  </div>`;
}

export function DocsPage() {
  const s = useStore();
  const docs = s.docs;
  const [page, setPage] = useState('getting-started');
  if (!docs) return html`<div class="splash"><${Spinner} /></div>`;
  const current = docs.pages.find((p) => p.id === page);
  const onClick = (e) => {
    const a = e.target.closest('a[data-link]');
    if (!a) return;
    e.preventDefault();
    const url = a.dataset.link;
    if (/^https?:/.test(url)) call('open_url', { url }).catch(() => {});
    else {
      const id = url.replace(/^.*\//, '').replace(/\.md.*$/, '');
      if (docs.pages.some((p) => p.id === id)) setPage(id);
    }
  };
  return html`<div class="docs">
    <nav class="docs-nav">
      <div class="nav-title" style="padding:0 12px 6px">Guides</div>
      ${docs.pages.map((p) => html`<button class=${page === p.id ? 'on' : ''} onClick=${() => setPage(p.id)}>${p.title}</button>`)}
      <hr />
      <div class="nav-title" style="padding:0 12px 6px">Reference</div>
      <button class=${page === 'api' ? 'on' : ''} onClick=${() => setPage('api')}>Plugin functions and events</button>
      <button class="" onClick=${() => call('open_url', { url: 'https://openomsi-org.github.io/openOMSI-Development-Tools/' })}>Online documentation <${Icon} name="external" size=${14} /></button>
    </nav>
    <div onClick=${onClick}>${page === 'api' ? html`<${Reference} api=${docs.api} />` : html`<div class="md" dangerouslySetInnerHTML=${{ __html: renderMarkdown(current?.markdown || '') }}></div>`}</div>
  </div>`;
}
