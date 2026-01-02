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
    pub fn resolve(key: &str) -> anyhow::Result<Secret<String>> {
        // Placeholder for MVP resolution
        // 1. Try Env Var
        if let Ok(v) = std::env::var(key) {
            return Ok(Secret::new(v));
        }

        // 2. Future: Call Wasm Plugins via WasmPluginEngine

        Ok(Secret::new(format!("resolved-{}", key)))
    }
}
