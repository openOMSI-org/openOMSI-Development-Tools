# Patched crates

`glib/` is glib 0.18.5 from crates.io (MIT, see its LICENSE and COPYRIGHT), the version of the
GTK3 stack that Tauri's Linux web view (webkit2gtk 4.1) is built on. glib 0.20, which fixes
RUSTSEC-2024-0429, belongs to the GTK4 bindings and cannot replace it. The one change:
`VariantStrIter::impl_get` (src/variant_iter.rs) passes the string pointer that
`g_variant_get_child` writes as `&mut p` of a mutable local, as glib 0.20 does; before, it wrote
through a pointer to an immutable local, which is undefined behaviour.

The openOMSI launcher carries the same patch. Drop it (and the `glib` line of `[patch.crates-io]`
in the root `Cargo.toml`) once Tauri moves to GTK4.
