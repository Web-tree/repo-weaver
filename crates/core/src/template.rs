use tera::{Context, Tera};

pub struct TemplateEngine {
    _tera: Tera,
}

impl TemplateEngine {
    pub fn new() -> anyhow::Result<Self> {
        let tera = Tera::default();
        Ok(Self { _tera: tera })
    }

    pub fn render(&self, template_str: &str, context: &Context) -> anyhow::Result<String> {
        // Use one_off for ad-hoc templates to avoid mutability requirement
        // TODO: Optimize if performance becomes an issue
        Ok(Tera::one_off(template_str, context, false)?)
    }
}
