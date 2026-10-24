// `OPENOMSI_DEVTOOLS_SELFTEST=<scratch folder>`: the window stays hidden and the page checks
// itself against the real commands and its own pages - a plugin made, edited, checked, built,
// signed, installed into a made-up game folder and inspected; every page drawn - then reports
// to the program, which prints it and ends (exit code 1 when a check failed).

import { S, set } from './store.js';
import { call, inTauri } from './api.js';
import { openProject, runBuild, saveTab, openFile, setTabText, refreshProblems, saveSettings, refreshBoot } from './data.js';

const wait = (ms) => new Promise((r) => setTimeout(r, ms));

export async function selftest() {
  const passed = [];
  const failed = [];
  let last = 'start';
  const say = (line) => call('selftest_log', { line }).catch(() => {});
  addEventListener('error', (e) => say('PAGE ERROR ' + e.message + ' at ' + e.filename + ':' + e.lineno));
  addEventListener('unhandledrejection', (e) => say('PAGE REJECTION ' + (e.reason?.stack || e.reason)));
  // (a check that never answers ends the run with a report instead of hanging it)
  setTimeout(() => call('selftest_done', { report: { failed: [`timeout after: ${last}`], passed: passed.length } }), 120000);
  const check = async (name, f) => {
    last = name;
    say('begin ' + name);
    try {
      const ok = await f();
      (ok === false ? failed : passed).push(ok === false ? `${name}: false` : name);
    } catch (e) {
      failed.push(`${name}: ${e.message || e}`);
    }
    say((failed[failed.length - 1]?.startsWith(name) ? 'FAILED ' : 'ok ') + name);
  };
  const root = S.boot.selftest.replace(/[\\/]+$/, '');
  const sep = S.boot.os === 'windows' ? '\\' : '/';
  const at = (...p) => [root, ...p].join(sep);

  say('boot ' + JSON.stringify(S.boot));
  await check('the page runs inside the program', () => inTauri);
  await check('the font is loaded', async () => (await document.fonts.load('14px Inter'), document.fonts.check('14px Inter')));
  await check('boot knows the version and the API', () => /^\d+\.\d+\.\d+/.test(S.boot.version) && /^\d+\.\d+/.test(S.boot.api_version));

  let dir = '';
  await check('make_id', async () => (await call('make_id', { author: 'Ann Lee', name: 'Stop Announcer' })) === 'com.ann_lee.stop_announcer');
  await check('a plugin is made', async () => {
    dir = await call('project_create', { req: { folder: at('plugins'), name: 'Selftest Panel', id: '', author: 'tester', kind: 'lua', template: 'panel' } });
    return dir.endsWith('selftest-panel');
  });
  await check('the project opens and the editor starts', async () => {
    await openProject(dir);
    await wait(900);
    return S.project.summary.id === 'com.tester.selftest_panel' && !!document.querySelector('.cm-editor') && document.querySelectorAll('.cm-line').length > 5;
  });
  await check('the tree lists the files', () => S.project.tree.some((n) => n.name === 'main.lua') && S.project.tree.some((n) => n.name === 'assets'));
  await check('a syntax error is found with its line', async () => {
    const p = await call('check_lua', { file: 'main.lua', text: 'local x = 1\nif x then\n' });
    return p.length > 0 && p[0].line >= 1 && p[0].severity === 'error';
  });
  await check('good Lua has no problems', async () => (await call('check_lua', { file: 'main.lua', text: 'local x = 1\nreturn x\n' })).length === 0);
  await check('a missing permission is named, with its fix', async () => {
    await openFile('main.lua');
    const text = S.tabs.find((t) => t.path === 'main.lua').text + '\nomsi.storage.set("a", 1)\n';
    setTabText('main.lua', text);
    await saveTab('main.lua', true);
    refreshProblems();
    await wait(900);
    const p = S.problems.find((x) => x.fix_permission === 'storage');
    return !!p && p.file === 'main.lua' && p.line > 1;
  });
  await check('the manifest saves and a bad one is refused', async () => {
    const m = structuredClone(S.project.manifest);
    m.plugin.permissions = ['ui', 'storage'];
    await call('manifest_save', { dir, manifest: m });
    const bad = structuredClone(m);
    bad.plugin.id = 'Not An Id!';
    let refused = false;
    try {
      await call('manifest_save', { dir, manifest: bad });
    } catch {
      refused = true;
    }
    refreshProblems();
    await wait(600);
    return refused && !S.problems.some((x) => x.severity === 'error');
  });
  await check('files are made, renamed and deleted inside the project only', async () => {
    await call('file_create', { dir, path: 'lib/util.lua', folder: false });
    await call('file_write', { dir, path: 'lib/util.lua', text: 'return {}' });
    await call('file_rename', { dir, from: 'lib/util.lua', to: 'lib/tools.lua' });
    const t = await call('project_tree', { dir });
    let outside = false;
    try {
      await call('file_read', { dir, path: '../secret' });
    } catch {
      outside = true;
    }
    await call('file_delete', { dir, path: 'lib' });
    return outside && t.some((n) => n.name === 'lib');
  });
  await check('a signing key is made', async () => {
    const k = await call('key_create', { label: 'Selftest key' });
    await refreshBoot();
    return /^[0-9a-f]{4}(:[0-9a-f]{4}){3}$/.test(k.fingerprint) && S.boot.keys.length === 1 && !!S.settings.default_key;
  });
  let built = null;
  await check('the plugin builds, signed', async () => {
    await saveSettings({ sign_builds: true });
    built = await runBuild(true);
    return built.size > 100 && !!built.signed_by;
  });
  await check('the built file inspects: valid signature, no source in the file list names', async () => {
    const r = await call('inspect', { path: built.out });
    return r.signed && r.valid && r.header.id === 'com.tester.selftest_panel' && r.files.some((f) => f.path === 'main.lua') && r.intact;
  });
  await check('an unsigned file can be signed', async () => {
    const plain = await call('build', { dir, sign: false, out: at('plain.oop') });
    const before = await call('inspect', { path: plain.out });
    const fp = await call('sign_file', { file: plain.out, key: S.settings.default_key });
    const after = await call('inspect', { path: plain.out });
    return !before.signed && after.valid && after.signer === fp;
  });
  await check('the game folder is checked', async () => {
    const game = at('game');
    await call('file_create', { dir: root, path: 'game/.openomsi-content', folder: false });
    const ok = await call('game_info', { path: game });
    const no = await call('game_info', { path: root });
    return ok.ok && ok.plugins.endsWith('plugins') && !no.ok;
  });
  await check('Run puts the Lua plugin into the game and keeps it up to date', async () => {
    await saveSettings({ game_folder: at('game') });
    const r = await call('run_in_game', { dir, launch: false });
    const first = await call('file_read', { dir: r.target, path: 'main.lua' });
    const again = await call('run_in_game', { dir, launch: false });
    return r.mode === 'folder' && first.text.includes('omsi.ui.set') && again.changed === 0;
  });
  await check('Run without a game folder asks for it', async () => {
    await saveSettings({ game_folder: '' });
    try {
      await call('run_in_game', { dir, launch: false });
      return false;
    } catch (e) {
      return e.message.startsWith('needs-game:');
    }
  });
  await check('a Rust plugin can be made', async () => {
    const d = await call('project_create', { req: { folder: at('plugins'), name: 'Selftest Rust', id: '', author: 'tester', kind: 'rust', template: 'hello' } });
    const r = await call('project_open', { dir: d });
    return r.summary.kind === 'rust' && r.tree.some((n) => n.name === 'src');
  });
  await check('the cargo check answers', async () => typeof (await call('cargo_info', { path: '' })).found === 'boolean');
  await check('the docs are packed', async () => {
    const d = await call('docs');
    return d.pages.length === 5 && d.api.functions.length > 100 && d.permissions.length >= 8;
  });
  await check('an update is found and a development build is not replaced', async () => {
    const r = await call('update_check');
    if (!r) return true; // (no test release described: the look itself worked)
    // (a packaged copy is replaced by the check below, once, and only when asked)
    if (!S.boot.dev_build) return true;
    let refused = '';
    try {
      await call('update_install', { release: r });
    } catch (e) {
      refused = e.message;
    }
    return /development build/.test(refused);
  });
  // (only for a copy of the packaged app, started with OPENOMSI_DEVTOOLS_SELFTEST_INSTALL=1)
  if (S.boot.selftest_install) {
    await check('the update downloads, is checked and replaces the running program', async () => {
      const exe = S.boot.exe;
      const parts = exe.split(/[\\/]/);
      const mac = S.boot.os === 'macos';
      const dir = (mac ? parts.slice(0, -1) : parts.slice(0, -1)).join(sep);
      const name = parts[parts.length - 1];
      const before = (await call('file_read', { dir, path: name })).size;
      const release = await call('update_check');
      if (!release) return false;
      await call('update_install', { release });
      const after = (await call('file_read', { dir, path: name })).size;
      return after !== before && !S.boot.dev_build;
    });
  }
  for (const page of ['start', 'work', 'details', 'build', 'keys', 'inspector', 'docs', 'settings']) {
    await check(`the ${page} page draws`, async () => {
      set({ page });
      await say('page set');
      await wait(450);
      await say('waited');
      const el = document.querySelector('.page');
      return !!el && el.textContent.trim().length > 20 && !document.querySelector('.notice.danger');
    });
  }

  const report = { version: S.boot.version, passed: passed.length, failed, checks: passed };
  await call('selftest_done', { report });
}
