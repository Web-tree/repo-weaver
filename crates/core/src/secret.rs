use crate::config::SecretConfig;
use crate::plugin::resolver::PluginResolver;
use crate::plugin::wasm::WasmPluginEngine;
use std::fmt;

#[derive(Clone)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

pub struct SecretResolver;

impl SecretResolver {
    /// Resolve a declared secret to its value.
    ///
    /// The `env` provider reads an environment variable named by `cfg.key`.
    /// Any other provider name is resolved as a secrets-provider plugin
    /// (e.g. `aws-ssm`) via the WASM `provider` world, which runs the
    /// underlying native tool (`aws ssm get-parameter`, ...) inside the
    /// sandbox.
    pub async fn resolve(
        cfg: &SecretConfig,
        plugin_resolver: &PluginResolver,
    ) -> anyhow::Result<Secret<String>> {
        if cfg.provider == "env" {
            let value = std::env::var(&cfg.key)
                .map_err(|_| anyhow::anyhow!("secret env var '{}' is not set", cfg.key))?;
            return Ok(Secret::new(value));
        }

        // Provider name maps to a plugin name (e.g. "aws-ssm").
        let resolved = plugin_resolver
            .resolve_ensure_type(&cfg.provider)
            .await
            .map_err(|e| {
                anyhow::anyhow!("failed to resolve secrets provider '{}': {e}", cfg.provider)
            })?;

        let engine = WasmPluginEngine::new()?;
        let provider = engine.load_provider(&resolved.wasm_path)?;
        let value = provider.get_secret(&cfg.key)?;
        Ok(Secret::new(value))
    }
}
