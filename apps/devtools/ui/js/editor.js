// One file of the project in a CodeMirror editor (the bundle in vendor/editor.js). Every open
// file keeps its own editor, hidden when it is not the one shown, so that undo, selection and
// scroll position stay with the file.

import { html, useEffect, useRef } from '../vendor/preact-htm.js';
import { createEditor, languageOf } from '../vendor/editor.js';
import { call } from './api.js';

export { languageOf };

export function EditorHost({ dir, tab, active, api, extra, goto, onText, onSave, onCursor }) {
  const host = useRef();
  const ed = useRef();
  const latest = useRef({});
  latest.current = { onText, onSave, onCursor };

  useEffect(() => {
    ed.current = createEditor({
      parent: host.current,
      text: tab.text,
      language: languageOf(tab.path),
      api,
      onChange: (t) => latest.current.onText(t),
      onSave: () => latest.current.onSave(),
      onCursor: (c) => latest.current.onCursor(c),
      lint: (text) => call('check_lua', { file: tab.path, text }),
    });
    return () => ed.current.destroy();
  }, []);
  // (the text was changed from outside the editor: a reload of the file)
  useEffect(() => {
    if (ed.current && ed.current.getText() !== tab.text) ed.current.setText(tab.text);
  }, [tab.rev]);
  const key = JSON.stringify(extra);
  useEffect(() => {
    ed.current?.setExtra(extra);
  }, [key]);
  useEffect(() => {
    if (active) ed.current?.focus();
  }, [active]);
  useEffect(() => {
    if (goto && goto.path === tab.path && active) ed.current?.goto(goto.line, goto.col);
  }, [goto?.n, active]);
  return html`<div class="editor-host" ref=${host} style=${active ? '' : 'display:none'}></div>`;
}
