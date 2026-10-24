// The code editor of the openOMSI Development Tools: CodeMirror 6 with what plugin authors need
// (Lua, Rust, TOML and JSON highlighting, completion and hover help for the plugin API, problems
// shown where they are). Bundled by `build.sh` into ../ui/vendor/editor.js; the page imports
// `createEditor` from there.

import { EditorState, Compartment } from '@codemirror/state';
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection, dropCursor, rectangularSelection, crosshairCursor, hoverTooltip, placeholder } from '@codemirror/view';
import { defaultKeymap, history, historyKeymap, indentWithTab, toggleComment } from '@codemirror/commands';
import { StreamLanguage, syntaxHighlighting, HighlightStyle, indentOnInput, bracketMatching, foldGutter, foldKeymap, indentUnit } from '@codemirror/language';
import { closeBrackets, closeBracketsKeymap, autocompletion, completionKeymap, completeFromList } from '@codemirror/autocomplete';
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
import { lintGutter, linter, setDiagnostics, lintKeymap } from '@codemirror/lint';
import { tags as t } from '@lezer/highlight';
import { lua } from '@codemirror/legacy-modes/mode/lua';
import { rust } from '@codemirror/legacy-modes/mode/rust';
import { toml } from '@codemirror/legacy-modes/mode/toml';
import { json } from '@codemirror/lang-json';

// colours come from the page's CSS variables, so the editor follows the light and dark look
const highlight = HighlightStyle.define([
  { tag: [t.keyword, t.controlKeyword, t.definitionKeyword, t.moduleKeyword, t.operatorKeyword], color: 'var(--syn-keyword)' },
  { tag: [t.string, t.special(t.string)], color: 'var(--syn-string)' },
  { tag: [t.number, t.bool, t.null, t.atom], color: 'var(--syn-number)' },
  { tag: [t.comment, t.lineComment, t.blockComment], color: 'var(--syn-comment)', fontStyle: 'italic' },
  { tag: [t.function(t.variableName), t.function(t.propertyName), t.definition(t.variableName)], color: 'var(--syn-function)' },
  { tag: [t.propertyName, t.attributeName], color: 'var(--syn-property)' },
  { tag: [t.typeName, t.className, t.namespace], color: 'var(--syn-type)' },
  { tag: [t.operator, t.punctuation, t.bracket], color: 'var(--syn-punct)' },
  { tag: [t.heading], color: 'var(--syn-keyword)', fontWeight: '600' },
  { tag: t.invalid, color: 'var(--danger)' },
]);

const theme = EditorView.theme({
  '&': { height: '100%', color: 'var(--text)', backgroundColor: 'var(--editor-bg)', fontSize: '13px' },
  '&.cm-focused': { outline: 'none' },
  '.cm-scroller': { fontFamily: 'var(--mono)', lineHeight: '1.6', overflow: 'auto' },
  '.cm-content': { caretColor: 'var(--accent)', padding: '8px 0' },
  '.cm-gutters': { backgroundColor: 'var(--editor-bg)', color: 'var(--muted)', border: 'none' },
  '.cm-activeLine': { backgroundColor: 'var(--editor-line)' },
  '.cm-activeLineGutter': { backgroundColor: 'var(--editor-line)', color: 'var(--text-2)' },
  '.cm-cursor': { borderLeftColor: 'var(--accent)', borderLeftWidth: '2px' },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection': { backgroundColor: 'var(--accent-soft) !important' },
  '.cm-selectionMatch': { backgroundColor: 'var(--surface-3)' },
  '.cm-matchingBracket': { backgroundColor: 'var(--surface-3)', outline: '1px solid var(--line-strong)' },
  '.cm-tooltip': { backgroundColor: 'var(--surface)', border: '1px solid var(--line-strong)', borderRadius: '6px', boxShadow: 'var(--shadow)', color: 'var(--text)' },
  '.cm-tooltip-autocomplete > ul': { fontFamily: 'var(--mono)', maxHeight: '260px' },
  '.cm-tooltip-autocomplete > ul > li': { padding: '3px 8px' },
  '.cm-tooltip-autocomplete > ul > li[aria-selected]': { backgroundColor: 'var(--accent-soft)', color: 'var(--text)' },
  '.cm-completionDetail': { color: 'var(--muted)', fontStyle: 'normal', marginLeft: '10px' },
  '.cm-completionInfo': { maxWidth: '360px', padding: '8px 10px', fontFamily: 'Inter, system-ui, sans-serif', fontSize: '12.5px', lineHeight: '1.45' },
  '.cm-diagnostic': { padding: '4px 8px', fontFamily: 'Inter, system-ui, sans-serif', fontSize: '12.5px' },
  '.cm-diagnostic-error': { borderLeft: '3px solid var(--danger)' },
  '.cm-diagnostic-warning': { borderLeft: '3px solid var(--warn)' },
  '.cm-lintRange-error': { backgroundImage: 'none', textDecoration: 'underline wavy var(--danger)', textUnderlineOffset: '3px' },
  '.cm-lintRange-warning': { backgroundImage: 'none', textDecoration: 'underline wavy var(--warn)', textUnderlineOffset: '3px' },
  '.cm-panels': { backgroundColor: 'var(--surface)', color: 'var(--text)' },
  '.cm-panels-bottom': { borderTop: '1px solid var(--line-strong)' },
  '.cm-searchMatch': { backgroundColor: 'var(--accent-soft)', outline: '1px solid var(--accent)' },
  '.cm-panel.cm-search input, .cm-panel.cm-search button': { fontFamily: 'Inter, system-ui, sans-serif', fontSize: '12.5px' },
  '.cm-panel.cm-search input': { background: 'var(--surface-2)', border: '0', borderRadius: '4px', color: 'var(--text)', padding: '3px 8px' },
  '.cm-panel.cm-search button': { background: 'var(--surface-2)', border: '0', borderRadius: '4px', color: 'var(--text)', padding: '3px 8px', cursor: 'pointer' },
  '.cm-foldPlaceholder': { backgroundColor: 'var(--surface-3)', border: 'none', color: 'var(--muted)' },
});

const languages = {
  lua: () => StreamLanguage.define(lua),
  rust: () => StreamLanguage.define(rust),
  toml: () => StreamLanguage.define(toml),
  json: () => json(),
  text: () => [],
};

export function languageOf(path) {
  const ext = (path.split('.').pop() || '').toLowerCase();
  return { lua: 'lua', rs: 'rust', toml: 'toml', json: 'json' }[ext] || 'text';
}

const KEYWORDS = ['and', 'break', 'do', 'else', 'elseif', 'end', 'false', 'for', 'function', 'goto', 'if', 'in', 'local', 'nil', 'not', 'or', 'repeat', 'return', 'then', 'true', 'until', 'while'].map((label) => ({ label, type: 'keyword', boost: -2 }));
const BUILTINS = ['print', 'pairs', 'ipairs', 'tostring', 'tonumber', 'type', 'pcall', 'error', 'require', 'select', 'next', 'math', 'string', 'table', 'os.time', 'os.date', 'os.clock'].map((label) => ({ label, type: 'function', boost: -1 }));

/** The completion and hover data from the API manifest (`{functions, events}`). */
function apiData(api) {
  const fns = (api?.functions || []).map((f) => {
    const params = (f.params || []).map((p) => (p.optional ? `[${p.name}]` : p.name)).join(', ');
    return {
      name: f.name,
      full: 'omsi.' + f.name,
      signature: `omsi.${f.name}(${params})`,
      doc: f.doc || '',
      returns: f.returns || '',
      permission: f.permission || null,
      since: f.since || '',
    };
  });
  const events = (api?.events || []).map((e) => ({ name: e.name, args: (e.args || []).join(', '), doc: e.doc || '', since: e.since || '' }));
  return { fns, events, byName: new Map(fns.map((f) => [f.name, f])), eventByName: new Map(events.map((e) => [e.name, e])) };
}

function docNode(f) {
  const root = document.createElement('div');
  root.className = 'cm-api-doc';
  const sig = document.createElement('div');
  sig.className = 'sig';
  sig.textContent = f.signature + (f.returns ? '  → ' + f.returns : '');
  root.appendChild(sig);
  const p = document.createElement('div');
  p.className = 'text';
  // (the manifest's docs mark code with backticks)
  p.innerHTML = f.doc.replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c])).replace(/`([^`]+)`/g, '<code>$1</code>');
  root.appendChild(p);
  const meta = document.createElement('div');
  meta.className = 'meta';
  meta.textContent = [f.permission ? `needs the "${f.permission}" permission` : 'no permission needed', f.since ? `since ${f.since}` : ''].filter(Boolean).join('  ·  ');
  root.appendChild(meta);
  return root;
}

function luaCompletion(data) {
  const apiOptions = data.fns.map((f) => ({
    label: f.full,
    type: 'function',
    detail: f.signature.slice(f.full.length),
    info: () => docNode(f),
    boost: 2,
  }));
  const eventOptions = data.events.map((e) => ({
    label: e.name,
    type: 'enum',
    detail: e.args ? `(${e.args})` : '',
    info: () => {
      const n = document.createElement('div');
      n.className = 'cm-api-doc';
      n.textContent = e.doc;
      return n;
    },
  }));
  const general = [...KEYWORDS, ...BUILTINS];
  return (context) => {
    // an event name in omsi.on("...")
    const ev = context.matchBefore(/omsi\.(?:on|off)\(\s*["'][\w]*/);
    if (ev) {
      const q = ev.text.search(/["']/) + 1;
      return { from: ev.from + q, options: eventOptions, validFor: /^\w*$/ };
    }
    const word = context.matchBefore(/[\w.]+/);
    if (!word || (word.from === word.to && !context.explicit)) return null;
    if (word.text.startsWith('omsi') || word.text === 'o' || word.text === 'om' || word.text === 'oms') {
      return { from: word.from, options: [...apiOptions, { label: 'omsi', type: 'namespace' }], validFor: /^[\w.]*$/ };
    }
    return { from: word.from, options: general, validFor: /^\w*$/ };
  };
}

function luaHover(data) {
  return hoverTooltip((view, pos) => {
    const line = view.state.doc.lineAt(pos);
    const text = line.text;
    const col = pos - line.from;
    const re = /omsi(?:\.[A-Za-z_]\w*)+/g;
    let m;
    while ((m = re.exec(text))) {
      const start = m.index;
      const end = start + m[0].length;
      if (col < start || col > end) continue;
      // the longest dotted name the API has that ends at or after the pointer
      const parts = m[0].slice(5).split('.');
      for (let n = parts.length; n >= 1; n--) {
        const name = parts.slice(0, n).join('.');
        const f = data.byName.get(name);
        const nameEnd = start + 5 + name.length;
        if (f && col <= nameEnd) {
          return { pos: line.from + start, end: line.from + nameEnd, above: true, create: () => ({ dom: docNode(f) }) };
        }
      }
    }
    // an event name in a string of omsi.on
    const ev = /omsi\.(?:on|off)\(\s*["'](\w+)["']/.exec(text);
    if (ev) {
      const name = ev[1];
      const from = text.indexOf(name, ev.index);
      const e = data.eventByName.get(name);
      if (e && col >= from && col <= from + name.length) {
        return {
          pos: line.from + from,
          end: line.from + from + name.length,
          above: true,
          create: () => {
            const n = document.createElement('div');
            n.className = 'cm-api-doc';
            const sig = document.createElement('div');
            sig.className = 'sig';
            sig.textContent = `event "${e.name}"` + (e.args ? ` (${e.args})` : '');
            const d = document.createElement('div');
            d.className = 'text';
            d.textContent = e.doc;
            n.append(sig, d);
            return { dom: n };
          },
        };
      }
    }
    return null;
  }, { hoverTime: 250 });
}

/**
 * Makes an editor in `parent`.
 * options: text, language ('lua' | 'rust' | 'toml' | 'json' | 'text'), api (the manifest),
 * onChange(text), onSave(), onCursor({line, col}), lint(text) -> Promise<problems>, readOnly.
 */
export function createEditor(options) {
  const lang = new Compartment();
  const readOnly = new Compartment();
  const completion = new Compartment();
  const data = apiData(options.api);
  let extra = [];
  // (the syntax problems of the last run of the linter, kept to show beside the project's)
  let syntax = [];
  let language = options.language || 'text';

  const toDiagnostics = (state, problems) => {
    const out = [];
    for (const p of problems || []) {
      if (!p.line) continue;
      const doc = state.doc;
      const l1 = Math.min(Math.max(p.line, 1), doc.lines);
      const a = doc.line(l1);
      const from = Math.min(a.from + Math.max((p.col || 1) - 1, 0), a.to);
      const l2 = Math.min(Math.max(p.end_line || p.line, 1), doc.lines);
      const b = doc.line(l2);
      let to = Math.min(b.from + Math.max((p.end_col || p.col || 1) - 1, 0), b.to);
      if (to <= from) to = Math.min(from + 1, doc.length);
      out.push({ from, to, severity: p.severity === 'error' ? 'error' : 'warning', message: p.message });
    }
    return out;
  };

  const syntaxLinter = linter(
    async (view) => {
      if (language !== 'lua' || !options.lint) return toDiagnostics(view.state, extra);
      let found = [];
      try {
        found = await options.lint(view.state.doc.toString());
      } catch {
        found = [];
      }
      syntax = toDiagnostics(view.state, found);
      return [...syntax, ...toDiagnostics(view.state, extra)];
    },
    { delay: 450 },
  );

  const view = new EditorView({
    parent: options.parent,
    state: EditorState.create({
      doc: options.text || '',
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        foldGutter(),
        lintGutter(),
        history(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        indentUnit.of('  '),
        bracketMatching(),
        closeBrackets(),
        rectangularSelection(),
        crosshairCursor(),
        highlightActiveLine(),
        highlightSelectionMatches(),
        syntaxHighlighting(highlight),
        theme,
        placeholder(options.placeholder || ''),
        lang.of(languages[language]()),
        readOnly.of(EditorState.readOnly.of(!!options.readOnly)),
        completion.of(language === 'lua' ? [autocompletion({ override: [luaCompletion(data)], icons: false }), luaHover(data)] : []),
        syntaxLinter,
        keymap.of([
          { key: 'Mod-s', preventDefault: true, run: () => (options.onSave?.(), true) },
          { key: 'Mod-/', run: toggleComment },
          indentWithTab,
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...searchKeymap,
          ...historyKeymap,
          ...foldKeymap,
          ...completionKeymap,
          ...lintKeymap,
        ]),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) options.onChange?.(u.state.doc.toString());
          if (u.selectionSet || u.docChanged) {
            const head = u.state.selection.main.head;
            const line = u.state.doc.lineAt(head);
            options.onCursor?.({ line: line.number, col: head - line.from + 1 });
          }
        }),
      ],
    }),
  });

  return {
    view,
    getText: () => view.state.doc.toString(),
    setText(text) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text }, selection: { anchor: 0 } });
    },
    setLanguage(name) {
      language = name;
      view.dispatch({ effects: [lang.reconfigure(languages[name]()), completion.reconfigure(name === 'lua' ? [autocompletion({ override: [luaCompletion(data)], icons: false }), luaHover(data)] : [])] });
    },
    setReadOnly(on) {
      view.dispatch({ effects: readOnly.reconfigure(EditorState.readOnly.of(!!on)) });
    },
    /** Problems the project check found for this file (the syntax ones come from `lint`). */
    setExtra(problems) {
      extra = problems || [];
      view.dispatch(setDiagnostics(view.state, [...syntax, ...toDiagnostics(view.state, extra)]));
    },
    goto(line, col) {
      const doc = view.state.doc;
      const l = doc.line(Math.min(Math.max(line, 1), doc.lines));
      const pos = Math.min(l.from + Math.max((col || 1) - 1, 0), l.to);
      view.dispatch({ selection: { anchor: pos }, effects: EditorView.scrollIntoView(pos, { y: 'center' }) });
      view.focus();
    },
    insert(text) {
      const r = view.state.selection.main;
      view.dispatch({ changes: { from: r.from, to: r.to, insert: text }, selection: { anchor: r.from + text.length } });
      view.focus();
    },
    focus: () => view.focus(),
    destroy: () => view.destroy(),
  };
}
