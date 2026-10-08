//! The compiler must not change what a plugin does. Each corpus script sets a global `R` to a
//! string that captures its result; the test runs the original and every compiled form under the
//! same Lua 5.4 (mlua) and checks that `R` comes out identical, and that the compiled form really
//! was obfuscated (no original local names, no readable string literals).

use lua_obfuscator::{Module, Options, compile, compile_single};
use mlua::Lua;

fn run(src: &str) -> String {
    let lua = Lua::new();
    lua.load(src).exec().unwrap_or_else(|e| panic!("running failed: {e}\n--- source ---\n{src}"));
    lua.globals().get::<String>("R").expect("the script must set R")
}

fn check_all_levels(src: &str) {
    let expected = run(src);
    for level in [0u8, 1, 2, 3] {
        let out = compile_single(src, &Options::level(level)).unwrap_or_else(|e| panic!("compile failed: {e}"));
        let got = run(&out);
        assert_eq!(got, expected, "level {level} changed the result\n--- compiled ---\n{out}");
    }
}

const CORPUS: &[(&str, &str)] = &[
    (
        "closures_and_upvalues",
        r#"
        local function counter()
            local n = 0
            return function() n = n + 1 return n end
        end
        local c = counter()
        R = tostring(c() + c() * 10 + c() * 100)
    "#,
    ),
    (
        "upvalues_in_loops",
        r#"
        local fns = {}
        for i = 1, 3 do fns[i] = function() return i end end
        local s = 0
        for _, f in ipairs(fns) do s = s + f() end
        R = tostring(s)
    "#,
    ),
    (
        "varargs",
        r##"
        local function sum(...)
            local t = {...}
            local total = 0
            for i = 1, select("#", ...) do total = total + t[i] end
            return total, select("#", ...)
        end
        local a, b = sum(10, 20, 30)
        R = a .. "/" .. b
    "##,
    ),
    (
        "metatables",
        r#"
        local V = {}
        V.__index = V
        V.__add = function(a, b) return setmetatable({x = a.x + b.x}, V) end
        function V.new(x) return setmetatable({x = x}, V) end
        function V:double() return self.x * 2 end
        local p = V.new(3) + V.new(4)
        R = tostring(p.x) .. ":" .. tostring(p:double())
    "#,
    ),
    (
        "goto_and_labels",
        r#"
        local out = {}
        local i = 1
        ::top::
        if i <= 5 then
            if i % 2 == 0 then i = i + 1 goto top end
            out[#out + 1] = i
            i = i + 1
            goto top
        end
        R = table.concat(out, ",")
    "#,
    ),
    (
        "integer_float_semantics",
        r#"
        local a = 7 // 2
        local b = 7 / 2
        local c = 10 % 3
        local d = 2 ^ 10
        R = string.format("%d %.1f %d %d %s", a, b, c, math.type(d) == "float" and 1 or 0, tostring(d))
    "#,
    ),
    (
        "string_escapes",
        r#"
        local s = "tab\there\nnew\65\x42\u{43} 'q' \"dq\" back\\slash"
        R = tostring(#s) .. "|" .. s:gsub("%s", "_")
    "#,
    ),
    (
        "method_calls_and_self",
        r#"
        local obj = { items = {} }
        function obj:add(v) self.items[#self.items + 1] = v return self end
        function obj:total() local t = 0 for _, v in ipairs(self.items) do t = t + v end return t end
        R = tostring(obj:add(1):add(2):add(3):total())
    "#,
    ),
    (
        "multiple_assignment_swap",
        r#"
        local a, b, c = 1, 2, 3
        a, b, c = c, a, b
        local x, y = (function() return 10, 20 end)()
        R = string.format("%d%d%d-%d%d", a, b, c, x, y)
    "#,
    ),
    (
        "shadowing_and_local_equals_itself",
        r#"
        local x = 5
        local x = x + 1
        do local x = x * 10 R = tostring(x) end
        R = R .. "/" .. tostring(x)
    "#,
    ),
    (
        "numeric_for_step",
        r#"
        local t = {}
        for i = 10, 1, -3 do t[#t + 1] = i end
        R = table.concat(t, ",")
    "#,
    ),
    (
        "repeat_until_sees_local",
        r#"
        local i = 0
        repeat
            local done = i >= 3
            i = i + 1
        until done
        R = tostring(i)
    "#,
    ),
    (
        "table_keys_and_fields",
        r#"
        local k = "dynamic"
        local t = { a = 1, ["b"] = 2, [k] = 3, 10, 20 }
        R = string.format("%d%d%d%d%d", t.a, t.b, t.dynamic, t[1], t[2])
    "#,
    ),
    (
        "pcall_and_error",
        r#"
        local ok, err = pcall(function() error("boom") end)
        R = tostring(ok) .. ":" .. tostring(err):match("boom$")
    "#,
    ),
    (
        "string_library_calls",
        r#"
        local parts = {}
        for word in string.gmatch("the quick brown fox", "%a+") do parts[#parts + 1] = word:upper() end
        R = table.concat(parts, "-")
    "#,
    ),
];

#[test]
fn corpus_is_equivalent_at_every_level() {
    for (name, src) in CORPUS {
        std::panic::catch_unwind(|| check_all_levels(src)).unwrap_or_else(|_| panic!("corpus script {name:?} failed"));
    }
}

#[test]
fn output_is_actually_obfuscated() {
    let src = r#"
        local secretCounter = 0
        local function incrementTheCounter() secretCounter = secretCounter + 1 return secretCounter end
        R = tostring(incrementTheCounter()) .. "the password is swordfish"
    "#;
    let out = compile_single(src, &Options::level(2)).unwrap();
    assert!(!out.contains("secretCounter"), "a local name survived");
    assert!(!out.contains("incrementTheCounter"), "a local name survived");
    assert!(!out.contains("swordfish"), "a string literal survived in clear text");
    assert!(!out.contains("--"), "comments/formatting markers survived");
    // But the result is unchanged.
    assert_eq!(run(&out), run(src));
}

#[test]
fn modules_are_bundled_and_run() {
    let entry = r#"
        local util = require("lib.util")
        local greet = require("greet")
        R = util.shout(greet.hello("world"))
    "#;
    let modules = [
        Module {
            require_name: "lib.util".into(),
            source: "local M = {}\nfunction M.shout(s) return s:upper() .. \"!\" end\nreturn M\n".into(),
        },
        Module {
            require_name: "greet".into(),
            source: "local G = {}\nfunction G.hello(who) return \"hello \" .. who end\nreturn G\n".into(),
        },
    ];
    let out = compile(entry, &modules, &Options::level(2)).unwrap();
    assert_eq!(run(&out), "HELLO WORLD!");
    // Required twice would still load once: check the cache path by requiring again.
    let entry2 = r#"
        local a = require("m")
        local b = require("m")
        R = tostring(a == b) .. ":" .. tostring(a.n)
    "#;
    let modules2 =
        [Module { require_name: "m".into(), source: "COUNT = (COUNT or 0) + 1\nreturn { n = COUNT }\n".into() }];
    let out2 = compile(entry2, &modules2, &Options::level(1)).unwrap();
    assert_eq!(run(&out2), "true:1");
}

#[test]
fn api_globals_and_event_functions_keep_their_names() {
    let src = r#"
        local calls = {}
        function on_frame(dt) calls[#calls + 1] = dt end
        local function helper() return omsi_stub_value end
        omsi_stub_call(helper())
        R = "ok"
    "#;
    // Stub the globals the plugin uses so it runs here.
    let lua = Lua::new();
    lua.globals().set("omsi_stub_value", 42).unwrap();
    lua.globals().set("omsi_stub_call", lua.create_function(|_, _v: i64| Ok(())).unwrap()).unwrap();
    let out = compile_single(src, &Options::level(2)).unwrap();
    assert!(out.contains("on_frame"), "the event function was renamed");
    assert!(out.contains("omsi_stub_value"), "a global read was renamed");
    assert!(out.contains("omsi_stub_call"), "a global call was renamed");
    lua.load(&out).exec().unwrap();
}
