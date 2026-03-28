use crate::addr::AddrArgs;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Object {
    Addr(AddrArgs),
}

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub obj: Object,
}
