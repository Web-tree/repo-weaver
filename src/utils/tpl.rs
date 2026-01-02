use minijinja::Environment;

pub struct TemplateEngine {
    env: Environment<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let env = Environment::new();
        // Add default filters/globals here
        Self { env }
    }

    pub fn render(
        &self,
        template: &str,
        context: serde_json::Value,
    ) -> Result<String, minijinja::Error> {
        self.env.render_str(template, context)
    }
}
