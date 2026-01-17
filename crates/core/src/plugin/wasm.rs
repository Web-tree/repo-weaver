use std::io::Write;
use std::process::{Command, Stdio};
use wasmtime::component::{HasSelf, Linker, ResourceTable, bindgen};
use wasmtime::{Config, Engine};

bindgen!({
    world: "provider",
    path: "../../wit",
});

pub struct Host {
    // We might need WASI context later, but for now just custom interfaces
    pub table: ResourceTable,
}

impl Host {
    pub fn new() -> Self {
        Self {
            table: ResourceTable::new(),
        }
    }
}

impl weaver::plugin::process::Host for Host {
    fn exec(
        &mut self,
        req: weaver::plugin::process::ExecRequest,
    ) -> Result<weaver::plugin::process::ExecResult, String> {
        // Policy: Allowlist? For MVP, we allow everything but could restrict.
        // Spec says "Host can reject...". For now, we trust the plugin in MVP.

        let mut cmd = Command::new(&req.program);
        cmd.args(&req.args);

        if let Some(cwd) = &req.cwd {
            cmd.current_dir(cwd);
        }

        if !req.inherit_env {
            cmd.env_clear();
        }

        for (k, v) in &req.env {
            cmd.env(k, v);
        }

        cmd.stdin(if req.stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to spawn {}: {}", req.program, e)),
        };

        if let Some(input) = &req.stdin {
            if let Some(mut s) = child.stdin.take() {
                if let Err(e) = s.write_all(input) {
                    return Err(format!("Failed to write stdin: {}", e));
                }
            }
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Failed to wait on child: {}", e))?;
        let code = output.status.code().unwrap_or(1) as u32;

        Ok(weaver::plugin::process::ExecResult {
            status: code,
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

pub struct WasmPluginEngine {
    engine: Engine,
    linker: Linker<Host>,
}

impl WasmPluginEngine {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        let mut linker = Linker::new(&engine);

        // Link our world
        Provider::add_to_linker::<Host, HasSelf<Host>>(&mut linker, |state| state)?;

        Ok(Self { engine, linker })
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    // Helper to instantiate and run "get-secret" if we wanted to call it from here.
    // For now, T022 manually calls plugins.
    // We need to expose a way to get the Linker or instantiate.

    pub fn linker(&self) -> &Linker<Host> {
        &self.linker
    }
}
