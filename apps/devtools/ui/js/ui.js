// The page's controls and the dialogs every page uses.

import { html, useEffect, useRef, useState } from '../vendor/preact-htm.js';
import { Icon } from './icons.js';
import { S, set } from './store.js';

export { html };

export function Button({ kind = '', icon, children, label, onClick, disabled, title, type = 'button', big, wide, small }) {
  const cls = ['btn', kind, big && 'big', wide && 'wide', small && 'small', icon && !children && !label && 'icon'].filter(Boolean).join(' ');
  return html`<button type=${type} class=${cls} onClick=${onClick} disabled=${disabled} title=${title}>
    ${icon && html`<${Icon} name=${icon} />`}${label || children}
  </button>`;
}

export function Card({ title, hint, actions, children, class: cls = '', body = true }) {
  return html`<section class=${'card ' + cls}>
    ${(title || actions) && html`<div class="card-head"><div><h2>${title}</h2>${hint && html`<div class="hint">${hint}</div>`}</div>${actions && html`<div class="row">${actions}</div>`}</div>`}
    ${body ? html`<div class="card-body">${children}</div>` : children}
  </section>`;
}

export function Field({ label, help, children, class: cls = '' }) {
  return html`<div class=${'field ' + cls}>${label && html`<label>${label}</label>`}${children}${help && html`<div class="help">${help}</div>`}</div>`;
}

export function Input({ value, onInput, mono, placeholder, disabled, onEnter, autofocus, spellcheck = false }) {
  const ref = useRef();
  useEffect(() => {
    if (autofocus) ref.current?.focus();
  }, []);
  return html`<input ref=${ref} class=${'input' + (mono ? ' mono' : '')} value=${value ?? ''} placeholder=${placeholder} disabled=${disabled} spellcheck=${spellcheck}
    onInput=${(e) => onInput?.(e.currentTarget.value)} onKeyDown=${(e) => e.key === 'Enter' && onEnter?.()} />`;
}

/** A select of `options`: [value, label] pairs. */
export function Select({ value, options, onChange, disabled }) {
  const now = String(value ?? '');
  const has = options.some(([v]) => String(v) === now);
  return html`<select class="select" value=${now} disabled=${disabled} onChange=${(e) => onChange(e.currentTarget.value)}>
    ${!has && html`<option value=${now}>${now || 'Choose…'}</option>`}
    ${options.map(([v, l]) => html`<option value=${String(v)} selected=${String(v) === now}>${l}</option>`)}
  </select>`;
}

export function Toggle({ checked, onChange, title, sub, disabled }) {
  return html`<label class=${'toggle' + (disabled ? ' disabled' : '')}>
    <input type="checkbox" checked=${!!checked} disabled=${disabled} onChange=${(e) => onChange(e.currentTarget.checked)} />
    <span class="track"></span>
    ${title && html`<span class="text"><b>${title}</b>${sub && html`<span>${sub}</span>`}</span>`}
  </label>`;
}

/** Choose one of `options`: [value, label] pairs. */
export function Segmented({ value, options, onChange, full }) {
  return html`<div class=${'segmented' + (full ? ' full' : '')}>
    ${options.map(([v, l]) => html`<button type="button" class=${v === value ? 'on' : ''} onClick=${() => onChange(v)}>${l}</button>`)}
  </div>`;
}

export function Spinner() {
  return html`<span class="spinner"></span>`;
}

export function Tag({ kind = '', children, title }) {
  return html`<span class=${'tag ' + kind} title=${title}>${children}</span>`;
}

export function Notice({ kind = '', icon = 'info', children }) {
  return html`<div class=${'notice ' + kind}><${Icon} name=${icon} /><div class="grow selectable">${children}</div></div>`;
}

export function Empty({ icon = 'folder_open', title, children }) {
  return html`<div class="empty"><${Icon} name=${icon} size=${36} /><b>${title}</b><div>${children}</div></div>`;
}

export function Toasts({ list }) {
  return html`<div class="toasts">${list.map((x) => html`<div class=${'toast ' + x.kind}><${Icon} name=${x.kind === 'error' ? 'error' : x.kind === 'ok' ? 'ok' : 'info'} /><div class="selectable">${x.text}</div></div>`)}</div>`;
}

// --- dialogs ---------------------------------------------------------------------------

/** Shows `Comp` over the page; it gets `close(value)`. Resolves with what it closes with. */
export function openDialog(Comp, props = {}) {
  return new Promise((resolve) => {
    set({ dialog: { Comp, props, resolve } });
  });
}

export function DialogHost() {
  const d = S.dialog;
  if (!d) return null;
  const close = (v) => {
    set({ dialog: null });
    d.resolve(v);
  };
  return html`<div class="overlay"><${d.Comp} ...${d.props} close=${close} /></div>`;
}

export function Dialog({ title, children, foot, close, wide }) {
  useEffect(() => {
    const k = (e) => e.key === 'Escape' && close(null);
    addEventListener('keydown', k);
    return () => removeEventListener('keydown', k);
  }, []);
  return html`<div class="dialog" role="dialog" aria-label=${title} style=${wide ? 'width:min(720px,100%)' : ''}>
    <div class="dialog-head"><h2>${title}</h2><${Button} kind="ghost" small icon="x" onClick=${() => close(null)} title="Close" /></div>
    <div class="dialog-body">${children}</div>
    ${foot && html`<div class="dialog-foot">${foot}</div>`}
  </div>`;
}

function PromptDialog({ title, label, value, confirm = 'OK', help, close }) {
  const [text, setText] = useState(value || '');
  const go = () => text.trim() && close(text.trim());
  return html`<${Dialog} title=${title} close=${close} foot=${html`<${Button} label="Cancel" onClick=${() => close(null)} /><${Button} kind="primary" label=${confirm} disabled=${!text.trim()} onClick=${go} />`}>
    <${Field} label=${label} help=${help}><${Input} value=${text} onInput=${setText} onEnter=${go} autofocus /></${Field}>
  </${Dialog}>`;
}

/** Asks for a line of text; null when cancelled. */
export const ask = (props) => openDialog(PromptDialog, props);

function ConfirmDialog({ title, text, confirm = 'OK', danger, close }) {
  return html`<${Dialog} title=${title} close=${close} foot=${html`<${Button} label="Cancel" onClick=${() => close(false)} /><${Button} kind=${danger ? 'danger' : 'primary'} label=${confirm} onClick=${() => close(true)} />`}>
    <p class="dim">${text}</p>
  </${Dialog}>`;
}

/** Asks to confirm; true or false. */
export const confirmBox = async (props) => (await openDialog(ConfirmDialog, props)) === true;

/** Copies text to the clipboard. */
export async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}

export const fmtSize = (n) => (n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(n < 10240 ? 1 : 0)} KiB` : `${(n / 1048576).toFixed(1)} MiB`);

/** The last part of a path. */
export const baseName = (p) => (p || '').split(/[\\/]/).filter(Boolean).pop() || p;

/** A path short enough for a line: its end. */
export function shortPath(p, max = 52) {
  if (!p || p.length <= max) return p || '';
  return '…' + p.slice(p.length - max + 1);
}
