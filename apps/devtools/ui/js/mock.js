// The commands answered inside the page, for a look at the layout in a browser (serve the
// repository's root and open /apps/devtools/ui/): a project of made-up files, nothing on disk.

const listeners = new Map();
const emit = (event, payload) => (listeners.get(event) || []).forEach((f) => f(payload));
export function listen(event, fn) {
  listeners.set(event, [...(listeners.get(event) || []), fn]);
  return () => listeners.set(event, (listeners.get(event) || []).filter((f) => f !== fn));
}

const DIR = '/Users/you/openOMSI plugins/stop-announcer';
const files = {
  'main.lua': `-- Announces each new stop and counts them across sessions.
local count = omsi.storage.get("stops") or 0

omsi.on("start", function()
  omsi.message("Stop Announcer is on. Stops so far: " .. count, 5)
end)

omsi.on("stop_arrive", function(name, id, number, delay)
  count = count + 1
  omsi.storage.set("stops", count)
  omsi.ui.toast(name .. "  (" .. omsi.fmt.delay(delay) .. ")", { seconds = 6 })
end)

omsi.every(1, function()
  local kmh = omsi.var("Velocity")
  if kmh and kmh > 50 then
    omsi.message(string.format("Slow down: %.0f km/h", kmh), 1)
  end
end)
`,
  'util.lua': 'local M = {}\nfunction M.round(x) return math.floor(x + 0.5) end\nreturn M\n',
  'assets/.keep': '',
  'openomsi-plugin.toml': '[plugin]\nid = "com.you.stop_announcer"\nname = "Stop Announcer"\nversion = "0.1.0"\nkind = "lua"\nentry = "main.lua"\nauthors = ["you"]\ndescription = "Announces each new bus stop."\npermissions = ["ui"]\n',
};
let manifest = {
  plugin: { id: 'com.you.stop_announcer', name: 'Stop Announcer', version: '0.1.0', kind: 'lua', entry: 'main.lua', authors: ['you'], description: 'Announces each new bus stop and counts them across sessions.', min_openomsi: '0.2.21', permissions: ['ui'], license: 'MIT' },
  lua: { obfuscation_level: 2 },
  rust: { features: [] },
};
let settings = { cargo_path: '', game_folder: '', theme: 'dark', update_mode: 'ask', default_key: '', sign_builds: false, author: 'you', projects_folder: '', recent: [DIR], skipped_version: '', live_in_game: true };
let keys = [];

const tree = () => {
  const root = [];
  for (const path of Object.keys(files).sort()) {
    if (path === '.gitignore') continue;
    const parts = path.split('/');
    let level = root;
    parts.forEach((name, i) => {
      const p = parts.slice(0, i + 1).join('/');
      if (i === parts.length - 1) {
        if (name !== '.keep') level.push({ name, path: p, dir: false, size: files[path].length, children: [] });
        return;
      }
      let d = level.find((n) => n.dir && n.name === name);
      if (!d) level.push((d = { name, path: p, dir: true, size: 0, children: [] }));
      level = d.children;
    });
  }
  const sort = (l) => { l.sort((a, b) => b.dir - a.dir || a.name.localeCompare(b.name)); l.forEach((n) => sort(n.children)); };
  sort(root);
  return root;
};
const summary = () => ({ dir: DIR, name: manifest.plugin.name, id: manifest.plugin.id, version: manifest.plugin.version, kind: manifest.plugin.kind, entry: manifest.plugin.entry || 'main.lua', error: null });

const lua = (text) => {
  // (a stand-in for the real parser: an `end` too few or too many)
  const opens = (text.match(/\b(function|if|for|while|do)\b/g) || []).length - (text.match(/\bfor\b[^\n]*\bdo\b|\bwhile\b[^\n]*\bdo\b/g) || []).length + (text.match(/\brepeat\b/g) || []).length * 0;
  const ends = (text.match(/\bend\b/g) || []).length;
  if (opens > ends) return [{ file: 'main.lua', line: Math.max(1, text.split('\n').length), col: 1, end_line: Math.max(1, text.split('\n').length), end_col: 2, severity: 'error', message: "expected 'end' to close the block" }];
  return [];
};

const problems = () => {
  const out = [];
  for (const [f, t] of Object.entries(files)) if (f.endsWith('.lua')) out.push(...lua(t).map((p) => ({ ...p, file: f })));
  const text = files['main.lua'];
  text.split('\n').forEach((l, i) => {
    const m = /omsi\.storage\./.exec(l);
    if (m && !manifest.plugin.permissions.includes('storage')) out.push({ file: 'main.lua', line: i + 1, col: m.index + 1, end_line: i + 1, end_col: m.index + 14, severity: 'warning', message: 'omsi.storage.get needs the "storage" permission, which the plugin does not declare', fix_permission: 'storage' });
  });
  return out;
};

export async function call(cmd, args = {}) {
  await new Promise((r) => setTimeout(r, 30));
  switch (cmd) {
    case 'boot':
      return { version: '0.2.0', os: 'macos', dev_build: true, updates_disabled: false, selftest: null, settings, recent: [summary()], keys, cargo: { found: true, path: '/Users/you/.cargo/bin/cargo', version: 'cargo 1.97.1', wasm_target: false, rustup: true }, game: { ok: !!settings.game_folder, folder: settings.game_folder, program: '', plugins: settings.game_folder + '/plugins', problem: settings.game_folder ? '' : 'No folder chosen' }, home: '/Users/you', documents: '/Users/you/Documents', api_version: '0.2.22' };
    case 'save_settings':
      settings = { ...settings, ...args.patch };
      return settings;
    case 'project_open':
      return { summary: summary(), tree: tree(), manifest };
    case 'project_tree':
      return tree();
    case 'project_create':
      return DIR;
    case 'make_id':
      return `com.${(args.author || 'example').toLowerCase().replace(/\W+/g, '_')}.${(args.name || 'plugin').toLowerCase().replace(/\W+/g, '_')}`;
    case 'file_read':
      if (!(args.path in files)) throw new Error('cannot read ' + args.path);
      return { text: files[args.path], binary: false, size: files[args.path].length };
    case 'file_write':
      files[args.path] = args.text;
      return null;
    case 'file_create':
      files[args.path + (args.folder ? '/.keep' : '')] = '';
      return null;
    case 'file_delete':
      for (const k of Object.keys(files)) if (k === args.path || k.startsWith(args.path + '/')) delete files[k];
      return null;
    case 'file_rename':
      files[args.to] = files[args.from];
      delete files[args.from];
      return null;
    case 'manifest_save':
      manifest = args.manifest;
      return null;
    case 'check_project':
      return problems();
    case 'check_lua':
      return lua(args.text).map((p) => ({ ...p, file: args.file }));
    case 'needed_permissions':
      return { ui: 'ui.toast', ...(files['main.lua'].includes('omsi.storage') ? { storage: 'storage.get' } : {}) };
    case 'build':
      emit('build-log', { level: 'info', text: 'Building Stop Announcer 0.1.0 (Lua)' });
      emit('build-log', { level: 'info', text: 'compiled 2 Lua file(s) into main.lua (obfuscation level 2)' });
      emit('build-log', { level: 'good', text: `Built ${DIR}/dist/com.you.stop_announcer.oop (3 KiB)` });
      return { out: `${DIR}/dist/com.you.stop_announcer.oop`, size: 3270, notes: [], signed_by: args.sign ? '3f2a:9c01:77be:12d4' : null, ms: 420 };
    case 'run_in_game':
      if (!settings.game_folder) throw new Error('needs-game: No folder chosen');
      emit('build-log', { level: 'good', text: '2 file(s) updated in ' + settings.game_folder + '/plugins/com.you.stop_announcer' });
      return { mode: 'folder', target: settings.game_folder, changed: 2, launched: !!args.launch };
    case 'game_info':
      return { ok: /openomsi/i.test(args.path), folder: args.path, program: args.path + '/openOMSI.app', plugins: args.path + '/plugins', problem: /openomsi/i.test(args.path) ? '' : "openOMSI's program is not in this folder" };
    case 'find_game':
      return { ok: true, folder: '/Users/you/Downloads/openOMSI', program: '', plugins: '/Users/you/Downloads/openOMSI/plugins', problem: '' };
    case 'game_log':
      return ['[12:01:14] omsi::plugin: Lua plugin com.you.stop_announcer loaded', '[12:01:20] omsi::plugin: stop_announcer: Stops so far: 4', '[12:03:44] omsi::plugin: Lua plugin com.you.stop_announcer reloaded'];
    case 'pick_folder':
      return args.title.includes('openOMSI') ? '/Users/you/Downloads/openOMSI' : '/Users/you/Documents/openOMSI plugins';
    case 'pick_file':
    case 'pick_save':
      return null;
    case 'cargo_info':
      return { found: true, path: '/Users/you/.cargo/bin/cargo', version: 'cargo 1.97.1', wasm_target: true, rustup: true };
    case 'keys_list':
      return keys;
    case 'key_create':
      keys = [{ label: args.label, fingerprint: '3f2a:9c01:77be:12d4', path: '/Users/you/.config/openOMSI-devtools/keys/key.oopkey', public_path: '' }];
      settings.default_key = keys[0].path;
      return keys[0];
    case 'docs': {
      const ids = [['getting-started', 'Getting started'], ['oop-format', 'The .oop format'], ['cli', 'Command line (oopc)'], ['sdk', 'Rust SDK'], ['publishing', 'Publishing and signing']];
      const pages = await Promise.all(ids.map(async ([id, title]) => ({ id, title, markdown: await fetch(`/docs/${id}.md`).then((r) => r.text()).catch(() => '# ' + title) })));
      const api = await fetch('/crates/openomsi-plugin-sdk/api.json').then((r) => r.json()).catch(() => ({ functions: [], events: [], version: '0' }));
      return { pages, api, permissions: [{ name: 'ui', description: 'Show panels, notifications and messages on the screen' }, { name: 'storage', description: "Keep data between sessions in the plugin's own data folder and storage" }, { name: 'vehicle_write', description: "Change the player's bus: script variables, keys and triggers, doors, the destination" }, { name: 'audio', description: 'Play sounds' }] };
    }
    case 'update_check':
      return null;
    case 'reveal':
    case 'open_url':
    case 'write_text':
      return null;
    case 'inspect':
      return { path: args.path, header: { id: 'com.you.stop_announcer', name: 'Stop Announcer', version: '0.1.0', kind: 'lua', entry: 'main.lua', authors: ['you'], description: 'Announces each new stop.', permissions: ['ui', 'storage'], license: 'MIT', tool: 'oopc 0.2.0', created: '2026-10-23T18:00:00Z' }, format: 1, size: 3270, signed: true, valid: true, signer: '3f2a:9c01:77be:12d4', files: [{ path: 'main.lua', size: 1204 }], intact: true };
    default:
      throw new Error(`${cmd}: not in the mock`);
  }
}

/** `#work`, `#details` and so on: the page a picture of the layout is wanted of (docs screenshots). */
export async function showcase(hash, api) {
  const page = hash.replace('#', '');
  const { openProject, refreshBoot } = await import('./data.js');
  const { set } = await import('./store.js');
  if (['work', 'details', 'build', 'work-problems'].includes(page)) await openProject(DIR);
  if (page === 'keys' || page === 'build') {
    await call('key_create', { label: "Ann's key" });
    await refreshBoot();
  }
  if (page === 'build') set({ lastBuild: { out: `${DIR}/dist/com.you.stop_announcer.oop`, size: 3270, notes: [], signed_by: '3f2a:9c01:77be:12d4', ms: 420 }, settings: { ...api.S.settings, sign_builds: true } });
  if (page === 'inspector') set({ inspected: await call('inspect', { path: `${DIR}/dist/com.you.stop_announcer.oop` }) });
  if (page === 'docs') await api.loadDocs();
  set({ page: page === 'work-problems' ? 'work' : page });
}
