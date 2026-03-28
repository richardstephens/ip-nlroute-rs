use crate::addr::addr_main;
use crate::args::{Args, Object};
use crate::route::route_main;
use clap::Parser;

mod addr;
mod args;
mod route;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.obj {
        Object::Addr(args) => addr_main(args),
        Object::Route(args) => route_main(args),
    }?;

    Ok(())
}
