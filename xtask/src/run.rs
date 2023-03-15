use std::error::Error;
use duct::cmd;

pub fn script(name: &str) -> Result<(), Box<dyn Error>>{
    cmd!("cargo", "run", "--bin", name).run()?;

    Ok(())
}
