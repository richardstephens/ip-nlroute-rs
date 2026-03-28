use crate::addr::addr_main;
use crate::args::{Args, Object};
use clap::Parser;

mod addr;
mod args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.obj {
        Object::Addr(args) => addr_main(args),
    }?;

    Ok(())
}
