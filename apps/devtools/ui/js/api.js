// The commands of the app (src/main.rs) and its events. Outside the Tauri window (a browser
// opened on ui/ for a look at the layout) the commands are answered by mock.js.

const tauri = window.__TAURI__;
export const inTauri = !!tauri;

let mockModule = null;
async function mock() {
  mockModule ||= await import('./mock.js');
  return mockModule;
}

/** Call a command; rejects with the error text the command gave. */
export async function call(cmd, args = {}) {
  if (!inTauri) return (await mock()).call(cmd, args);
  try {
    return await tauri.core.invoke(cmd, args);
  } catch (e) {
    throw new Error(typeof e === 'string' ? e : e?.message || String(e));
  }
}

/** Listen to an event of the app's (returns the function that stops listening). */
export function on(event, fn) {
  if (!inTauri) {
    let stop = () => {};
    mock().then((m) => (stop = m.listen(event, fn)));
    return () => stop();
  }
  let stop = null;
  let gone = false;
  tauri.event.listen(event, (e) => fn(e.payload)).then((u) => {
    if (gone) u();
    else stop = u;
  });
  return () => {
    gone = true;
    if (stop) stop();
  };
}

/** Files dropped onto the window (paths), and whether something is being dragged over it. */
export function onDrop(fn, over) {
  if (!inTauri) return () => {};
  const w = tauri.webviewWindow?.getCurrentWebviewWindow?.() || tauri.window?.getCurrentWindow?.();
  if (!w?.onDragDropEvent) return () => {};
  let stop = null;
  w.onDragDropEvent((e) => {
    const p = e.payload;
    if (p.type === 'enter' || p.type === 'over') over?.(true);
    else if (p.type === 'leave') over?.(false);
    else if (p.type === 'drop') {
      over?.(false);
      fn(p.paths || []);
    }
  }).then((u) => (stop = u));
  return () => stop?.();
}
