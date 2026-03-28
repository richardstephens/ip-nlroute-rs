use clap::{Args, Subcommand};
use ip_nlroute::NetlinkRouteHandle;
use ip_nlroute::addr::get::AddrGetRequest;

#[derive(Args, Debug)]
pub struct AddrArgs {
    #[clap(subcommand)]
    sc: Option<AddrSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum AddrSubcommand {
    Show(AddrShowArgs),
}

#[derive(Args, Debug, Default)]
pub struct AddrShowArgs {
    #[clap(long)]
    ifname: Option<String>,
}

pub fn addr_main(args: AddrArgs) -> anyhow::Result<()> {
    match args.sc {
        None => addr_show(AddrShowArgs::default()),
        Some(AddrSubcommand::Show(args)) => addr_show(args),
    }
}
pub fn addr_show(sc_args: AddrShowArgs) -> anyhow::Result<()> {
    let mut nl = NetlinkRouteHandle::open()?;

    let req = match sc_args.ifname {
        None => AddrGetRequest::all(),
        Some(n) => AddrGetRequest::for_ifname(n.as_str())?,
    };

    let response = req.send(&mut nl)?;

    println!("{:#?}", response);

    Ok(())
}
