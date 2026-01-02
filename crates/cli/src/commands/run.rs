use clap::Args;

#[derive(Args)]
pub struct RunArgs {}

pub async fn run(_args: RunArgs) -> anyhow::Result<()> {
    Ok(())
}
