//! End-to-end tests that drive the built `oopc` binary like a user would.

use std::path::Path;
use std::process::Command;

fn oopc() -> Command {
    Command::new(env!("CARGO_BIN_EXE_oopc"))
}

fn write(dir: &Path, rel: &str, contents: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn lua_project(dir: &Path) {
    write(
        dir,
        "openomsi-plugin.toml",
        "[plugin]\nid = \"com.test.demo\"\nname = \"Demo\"\nversion = \"1.0.0\"\nkind = \"lua\"\nentry = \"main.lua\"\npermissions = [\"ui\"]\n",
    );
    write(
        dir,
        "main.lua",
        "local greeting_module = require(\"lib.text\")\nomsi.ui.toast(greeting_module.make(), { seconds = 3 })\n",
    );
    write(
        dir,
        "lib/text.lua",
        "local M = {}\nfunction M.make() local hidden_phrase = \"hello operator\" return hidden_phrase end\nreturn M\n",
    );
    write(dir, "assets/data.txt", "some asset bytes");
}

#[test]
fn new_build_inspect_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("demo");
    lua_project(&dir);

    let out = dir.join("demo.oop");
    let status = oopc().args(["build"]).arg(&dir).arg("-o").arg(&out).status().unwrap();
    assert!(status.success(), "build failed");

    let bytes = std::fs::read(&out).unwrap();
    let oop = oop_format::Oop::read(&bytes).expect("the built .oop reads back");
    assert_eq!(oop.header().id, "com.test.demo");
    assert_eq!(oop.header().kind, oop_format::Kind::Lua);

    // The module was bundled into the entry, the asset kept, and nothing is left in clear text.
    let paths: Vec<&str> = oop.files().iter().map(|(p, _)| p.as_str()).collect();
    assert!(paths.contains(&"main.lua"));
    assert!(paths.contains(&"assets/data.txt"));
    assert!(!paths.contains(&"lib/text.lua"), "a source module was shipped");
    let entry = String::from_utf8_lossy(oop.entry());
    assert!(!entry.contains("greeting_module"), "a local name survived: {entry}");
    assert!(!entry.contains("hidden_phrase"), "a local name survived: {entry}");
    assert!(!entry.contains("hello operator"), "a string survived: {entry}");
    // Table fields (the module's public API) and API calls are kept by design.
    assert!(entry.contains("toast"), "the API call was lost");
    assert!(entry.contains("make"), "the module's public field was lost");
}

#[test]
fn keygen_sign_verify() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("demo");
    lua_project(&dir);
    let key = tmp.path().join("k.oopkey");

    assert!(oopc().args(["keygen", "--label", "ci", "-o"]).arg(&key).status().unwrap().success());
    let out = dir.join("signed.oop");
    assert!(oopc().args(["build"]).arg(&dir).arg("-o").arg(&out).arg("--sign").arg(&key).status().unwrap().success());
    let verify = oopc().args(["verify"]).arg(&out).arg("--key").arg(key.with_extension("oopkey.pub")).output().unwrap();
    assert!(verify.status.success(), "verify failed: {}", String::from_utf8_lossy(&verify.stderr));

    // Tampering with the file must make verify fail.
    let mut bytes = std::fs::read(&out).unwrap();
    let n = bytes.len();
    bytes[n / 2] ^= 0xFF;
    let bad = dir.join("bad.oop");
    std::fs::write(&bad, &bytes).unwrap();
    assert!(!oopc().args(["verify"]).arg(&bad).status().unwrap().success(), "tampered file verified");
}

#[test]
fn check_reports_missing_permission() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("demo");
    // Uses set_var (needs vehicle_write) but declares nothing.
    write(
        dir.as_path(),
        "openomsi-plugin.toml",
        "[plugin]\nid = \"com.test.perm\"\nname = \"Perm\"\nversion = \"1.0.0\"\nkind = \"lua\"\nentry = \"main.lua\"\npermissions = []\n",
    );
    write(dir.as_path(), "main.lua", "omsi.set_var(\"x\", 1)\n");
    let out = oopc().args(["check"]).arg(&dir).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("vehicle_write"), "check did not flag the missing permission: {text}");
}

#[test]
fn rejects_bad_version() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("demo");
    write(
        dir.as_path(),
        "openomsi-plugin.toml",
        "[plugin]\nid = \"com.test.bad\"\nname = \"Bad\"\nversion = \"1.0\"\nkind = \"lua\"\nentry = \"main.lua\"\n",
    );
    write(dir.as_path(), "main.lua", "return\n");
    assert!(!oopc().args(["build"]).arg(&dir).status().unwrap().success(), "accepted a bad version");
}
