use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Object {
    Addr,
}

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(subcommand)]
    pub obj: Object,
}
