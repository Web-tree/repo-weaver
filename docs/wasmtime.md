# Wasmtime + Component Model (WIT) + WASI Preview 2 (WASI 0.2)

This is the most “standard” approach today for a **CLI plugin system** that can grow into **polyglot plugins** (Rust now, other languages later) using a stable interface definition language (**WIT**) and a widely used runtime (**Wasmtime**). ([Component Model][1])

---

## 1) What you build

* **Host CLI (Rust)** embeds Wasmtime and loads plugins as **WebAssembly Components** (`.wasm` component files).
* **Plugin API** is a WIT “world” (imports/exports contract).
* **Plugins** are built per language toolchain into a component:

  * Rust: `cargo component`
  * Python: `componentize-py`
  * Go: TinyGo + component tooling
  * JavaScript: `jco` + componentize tooling

WIT is the contract. The component model + canonical ABI handles type-safe calls across component boundaries. ([GitHub][2])

---

## 2) Baseline versions (from latest docs/pages)

* `wasmtime` crate: **40.0.0** ([Docs.rs][3])
* `wasmtime-wasi` crate: **40.0.0** ([Docs.rs][4])
* `wit-bindgen` crate: **0.50.0** ([Docs.rs][5])
* `cargo-component`: **0.21.1** ([Crates][6])
* `wasm-tools`: releases include **v1.243.0** (Dec 2025) ([GitHub][7])

---

## 3) The interface: WIT “world” for a CLI plugin

Create `wit/plugin.wit`:

```wit
package my:cli;

world plugin {
  // Plugin entrypoint used by host:
  export run: func(args: list<string>) -> result<string, string>;
}
```

Notes:

* A `world` contains imports/exports (functions/interfaces). ([GitHub][2])
* For “real” CLI integration you can also align with standard WASI CLI worlds like `wasi:cli/command` (supported by `jco`). ([Component Model][8])

---

## 4) Rust plugin (guest) using `cargo-component`

### Install tooling

```bash
cargo install cargo-component --locked
rustup target add wasm32-wasip2
```

`cargo-component` is the standard Rust path for producing components today (still marked experimental). ([GitHub][9])

### Create plugin crate

```bash
cargo component new --lib my_plugin
cd my_plugin
```

Set `Cargo.toml` (minimum):

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[package.metadata.component]
package = "my:cli"

[dependencies]
wit-bindgen = "0.50.0"
```

Implement `src/lib.rs`:

```rust
use wit_bindgen::generate;

generate!({
    path: "wit",
    world: "plugin",
});

struct Impl;

impl exports::my::cli::plugin::Guest for Impl {
    fn run(args: Vec<String>) -> Result<String, String> {
        Ok(format!("plugin got args: {:?}", args))
    }
}

export!(Impl);
```

Build the component:

```bash
cargo component build
# output: target/wasm32-wasip2/debug/my_plugin.wasm  (path may vary)
```

`wit-bindgen`’s `generate!` macro is the standard Rust guest binding generator for WIT/component-model. ([Docs.rs][5])

---

## 5) Rust host (CLI) loading plugins with Wasmtime Component Model

### Dependencies (`Cargo.toml`)

```toml
[dependencies]
anyhow = "1"
wasmtime = "40.0.0"
wasmtime-wasi = "40.0.0"
```

Wasmtime’s component embedding API lives under `wasmtime::component`. ([Wasmtime][10])

### Host code (minimal, single export call)

```rust
use anyhow::Result;
use wasmtime::{Engine, Store};
use wasmtime::component::{Component, Linker};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

wasmtime::component::bindgen!({
    path: "wit",
    world: "plugin",
});

fn main() -> Result<()> {
    let engine = Engine::default();

    // Load compiled plugin component (.wasm component)
    let component = Component::from_file(&engine, "plugins/my_plugin.wasm")?;

    // WASI context (optional but common for plugins that want stdio/fs/etc)
    let wasi: WasiCtx = WasiCtxBuilder::new()
        .inherit_stdio()
        .build();

    let mut store = Store::new(&engine, wasi);

    let mut linker = Linker::new(&engine);

    // Add WASI (Preview2) to the linker (components)
    wasmtime_wasi::add_to_linker(&mut linker, |ctx: &mut WasiCtx| ctx)?;

    // Instantiate and call
    let (bindings, _) = Plugin::instantiate(&mut store, &component, &linker)?;
    let res = bindings.call_run(&mut store, &["a".into(), "b".into()])?;

    match res {
        Ok(out) => println!("{out}"),
        Err(err) => eprintln!("plugin error: {err}"),
    }

    Ok(())
}
```

For WASI preview support in Wasmtime, you use `wasmtime-wasi` (for components/WASIp2 see its `p2` support). ([Docs.rs][11])

### Typical plugin discovery (what you described)

* Put `.wasm` components in `./plugins/`
* Add a config like:

```toml
[[plugins]]
id = "my_plugin"
path = "plugins/my_plugin.wasm"
enabled = true
```

Then load/instantiate each plugin, optionally keep instances cached per command or per run.

---

## 6) Non-Rust plugins

### A) Python plugin via `componentize-py`

This is the most “official-feeling” Python path for components right now. ([GitHub][12])

Install:

```bash
pip install componentize-py
```

Given `hello.wit` (same structure as above), generate guest bindings:

```bash
componentize-py -d hello.wit -w hello bindings hello_guest
```

Write plugin `app.py` (example pattern from docs):

```python
import wit_world

class WitWorld(wit_world.WitWorld):
    def hello(self) -> str:
        return "Hello from Python plugin"
```

Build component:

```bash
componentize-py -d hello.wit -w hello componentize --stub-wasi app -o app.wasm
```

The upstream README includes a full working flow (guest bindings, componentize, and a Python host example). ([GitHub][12])

### B) Go plugin (TinyGo)

Bytecode Alliance component-model docs include a Go path; TinyGo versions listed there have WASI 0.2/component support. ([Component Model][13])

Also see `wit-bindgen-go` generator tooling in Bytecode Alliance Go modules. ([GitHub][14])

### C) JavaScript

For CLI/dev workflows, `jco` can run components and can transpile components to JS modules; it supports `wasi:cli/command`. ([Component Model][8])

---

## 7) Practical “docs set” to implement Option A end-to-end

These are the primary docs you’ll keep open while building:

1. **Component Model docs (concepts + language support)**

* Component Model intro + design rationale ([Component Model][15])
* Language support index ([Component Model][16])
* Rust language support ([Component Model][17])
* Go language support ([Component Model][13])
* Python language support ([Component Model][18])
* JavaScript language support ([Component Model][19])

2. **WIT specification (the contract you author)**

* WIT design doc ([GitHub][2])

3. **Wasmtime embedding APIs**

* `wasmtime::component` API docs ([Wasmtime][10])
* `wasmtime-wasi` crate docs (Preview1/Preview2; components in `p2`) ([Docs.rs][11])

4. **Tooling**

* `cargo-component` (Rust components) ([GitHub][9])
* `wasm-tools` (inspect/compose/convert; useful in CI) ([GitHub][7])
* `componentize-py` (Python components) ([GitHub][12])
* `jco` (JS toolchain for components) ([GitHub][20])

---

## 8) Minimal folder layout for a real project

```text
my_cli/
  wit/
    plugin.wit
  src/
    main.rs
  plugins/
    my_plugin.wasm
    other_plugin.wasm
  plugin.toml
```

Build pipeline:

* validate plugin components (optional: `wasm-tools component wit` / inspection)
* load plugins from config
* instantiate via Wasmtime component linker
* call `run(...)` (or a richer API: register commands, provide capabilities, etc.)

---

If you want the “Option A” API to feel like a real CLI plugin system (subcommands, completion, structured outputs), the next step is to evolve the WIT world to something like:

* `export commands() -> list<command>`
* `export run(cmd: string, args: list<string>) -> result<output, error>`
* `record output { stdout: string, stderr: string, exit_code: u32 }`

…and optionally align with standard WASI CLI worlds where it makes sense. ([Component Model][8])

[1]: https://component-model.bytecodealliance.org/design/why-component-model.html?utm_source=chatgpt.com "Why the Component Model?"
[2]: https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md?utm_source=chatgpt.com "WIT.md - WebAssembly/component-model"
[3]: https://docs.rs/wasmtime "wasmtime - Rust"
[4]: https://docs.rs/crate/wasmtime-wasi/latest?utm_source=chatgpt.com "wasmtime-wasi 40.0.0"
[5]: https://docs.rs/crate/wit-bindgen/latest?utm_source=chatgpt.com "wit-bindgen 0.50.0"
[6]: https://crates.io/crates/cargo-component?utm_source=chatgpt.com "cargo-component - crates.io: Rust Package Registry"
[7]: https://github.com/bytecodealliance/wasm-tools/releases?utm_source=chatgpt.com "Releases · bytecodealliance/wasm-tools"
[8]: https://component-model.bytecodealliance.org/running-components/jco.html?utm_source=chatgpt.com "jco - The WebAssembly Component Model"
[9]: https://github.com/bytecodealliance/cargo-component?utm_source=chatgpt.com "bytecodealliance/cargo-component"
[10]: https://docs.wasmtime.dev/api/wasmtime/component/index.html?utm_source=chatgpt.com "wasmtime::component - Rust"
[11]: https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/?utm_source=chatgpt.com "wasmtime_wasi - Rust"
[12]: https://github.com/bytecodealliance/componentize-py "GitHub - bytecodealliance/componentize-py: Tool for targetting the WebAssembly Component Model using Python"
[13]: https://component-model.bytecodealliance.org/language-support/go.html?utm_source=chatgpt.com "Go - The WebAssembly Component Model"
[14]: https://github.com/bytecodealliance/go-modules?utm_source=chatgpt.com "bytecodealliance/go-modules: WebAssembly, WASI, and ..."
[15]: https://component-model.bytecodealliance.org/?utm_source=chatgpt.com "The WebAssembly Component Model: Introduction"
[16]: https://component-model.bytecodealliance.org/language-support.html?utm_source=chatgpt.com "Creating Components - The WebAssembly ..."
[17]: https://component-model.bytecodealliance.org/language-support/rust.html?utm_source=chatgpt.com "Rust - The WebAssembly Component Model"
[18]: https://component-model.bytecodealliance.org/language-support/python.html?utm_source=chatgpt.com "Python - The WebAssembly Component Model"
[19]: https://component-model.bytecodealliance.org/language-support/javascript.html?utm_source=chatgpt.com "JavaScript - The WebAssembly Component Model"
[20]: https://github.com/bytecodealliance/jco?utm_source=chatgpt.com "JavaScript toolchain for WebAssembly Components"
