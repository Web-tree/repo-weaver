Key point: the plugin does **not** “inherit all env automatically.” Instead, the plugin passes a map of env vars it wants to set/override for the command, and the host merges them with the host’s base env (optionally filtered).

## Design

### Goals

* Plugin author experience: “call a command”
* Host controls execution (cwd, PATH, timeouts, allowlists)
* Plugin can define env vars per call (or once per plugin run)
* Works across languages because the ABI is WIT types (strings, lists, records)

### WIT interface

Define `exec.wit`:

```wit
package my:cli;

record exec_request {
  program: string,
  args: list<string>,
  cwd: option<string>,
  env: list<tuple<string, string>>, // plugin-defined env to set/override
  inherit_env: bool,                // whether base host env is included
  stdin: option<list<u8>>,          // optional: pass bytes
}

record exec_result {
  status: u32,
  stdout: list<u8>,
  stderr: list<u8>,
}

interface process {
  exec: func(req: exec_request) -> result<exec_result, string>;
}

world plugin {
  import process: process;
  export run: func(args: list<string>) -> result<string, string>;
}
```

This gives you:

* A single, universal API for all plugin languages.
* Explicit env behavior (`inherit_env` + `env` map).
* Room for policy: the host can reject program/args/cwd/env based on config.

## Host behavior (Rust + Wasmtime)

Host implements `process.exec` using `std::process::Command` and applies policy:

* **Allowlist/denylist** commands (recommended). Even if your product embraces “native commands,” you still want guardrails like:

  * allow any program by default (if you really want), but optionally:
  * deny `rm`, `curl`, `bash -c`, etc. depending on your threat model.
* **Env merge strategy**

  * If `inherit_env = true`: start from `std::env::vars_os()` (or a filtered subset), then apply plugin `env` overrides.
  * If `inherit_env = false`: start from empty env, apply plugin `env`, and maybe add a minimal PATH if you choose.
* **Secret leakage control**

  * If you do inherit env, plugins can read/use secrets indirectly by calling tools that read env. That’s the intended feature in your plan, but document it clearly.
* **Timeouts and output limits**

  * Prevent runaway commands and huge outputs.

A minimal host-side execution approach:

```rust
use std::process::{Command, Stdio};

fn run_exec(
    program: &str,
    args: &[String],
    cwd: Option<&str>,
    env: &[(String, String)],
    inherit_env: bool,
    stdin: Option<&[u8]>,
) -> Result<(u32, Vec<u8>, Vec<u8>), String> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    if !inherit_env {
        cmd.env_clear();
    }

    for (k, v) in env {
        cmd.env(k, v);
    }

    cmd.stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() });
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    if let Some(input) = stdin {
        if let Some(mut s) = child.stdin.take() {
            use std::io::Write;
            s.write_all(input).map_err(|e| e.to_string())?;
        }
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    let code = output.status.code().unwrap_or(1) as u32;

    Ok((code, output.stdout, output.stderr))
}
```

Then wrap it into the Wasmtime component import implementation (generated via `wasmtime::component::bindgen!`).

## Plugin author experience (Rust example)

From the plugin side, it becomes “call exec with env”:

```rust
// inside plugin run(...)
let env = vec![
  ("AWS_PROFILE".to_string(), "dev".to_string()),
  ("AWS_REGION".to_string(), "eu-west-1".to_string()),
];

let req = my::cli::process::ExecRequest {
  program: "aws".to_string(),
  args: vec!["sts".into(), "get-caller-identity".into(), "--output".into(), "json".into()],
  cwd: None,
  env,
  inherit_env: true,
  stdin: None,
};

let res = my::cli::process::exec(req).map_err(|e| e.to_string())??;

if res.status != 0 {
  return Err(String::from_utf8_lossy(&res.stderr).to_string());
}

Ok(String::from_utf8_lossy(&res.stdout).to_string())
```

This same pattern is implementable in other languages as long as they have component bindings.

## What about “plugin defines env once”?

Two clean patterns:

### Pattern A: “env overlay” returned by plugin

Add an optional exported function:

```wit
export env_overlay: func() -> list<tuple<string,string>>;
```

Host calls it once after instantiation and merges it into a per-plugin base env for future `exec`.

### Pattern B: “Context” handle

Make `process` return an opaque handle for session-scoped settings. This is heavier and usually unnecessary for a CLI.

## Important tradeoffs (and how to make it sane)

If your product goal is “plugins can execute native commands like a user,” accept these as explicit product constraints and mitigate with defaults:

* Default `inherit_env = true`, but allow host config to strip sensitive keys by prefix (e.g., `AWS_SECRET_ACCESS_KEY`, `GITHUB_TOKEN`).
* Provide structured results (exit code + stdout/stderr bytes) and recommend JSON output flags.
* Strongly recommend an allowlist per plugin in config for production use, even if dev mode is permissive.

## Minimum config model

```toml
[[plugins]]
id = "aws-tools"
path = "plugins/aws-tools.wasm"
exec = { inherit_env = true, allow = ["aws", "git", "terraform"] }
env_allow_prefixes = ["AWS_", "TF_", "MYAPP_"]   # optional: restrict what plugin can set
```
