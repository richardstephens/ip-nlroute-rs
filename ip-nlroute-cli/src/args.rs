use crate::addr::AddrArgs;
use crate::route::RouteArgs;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Object {
    Addr(AddrArgs),
    Route(RouteArgs),
}

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub obj: Object,
}
