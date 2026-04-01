use anyhow::Result;

fn main()->Result<()> {
let version = std::env::args().nth(1).unwrap_or("dev".to_string());
println!("preparing release {version}");
Ok(())
}
