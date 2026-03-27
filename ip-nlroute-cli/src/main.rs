use crate::addr::get_addresses;
use crate::args::{Args, Object};
use clap::Parser;

mod addr;
mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.obj {
        Object::Addr => {
            get_addresses()?;
        }
    }

    Ok(())
}
