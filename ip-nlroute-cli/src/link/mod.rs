use clap::{Args, Subcommand};
use ip_nlroute::NetlinkRouteHandle;
use ip_nlroute::link::LinkGetRequest;

#[derive(Args, Debug)]
pub struct LinkArgs {
    #[clap(subcommand)]
    sc: Option<LinkSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum LinkSubcommand {
    Show(LinkShowArgs),
}

#[derive(Args, Debug, Default)]
pub struct LinkShowArgs {
    #[clap(long)]
    ifname: Option<String>,
}

pub fn link_main(args: LinkArgs) -> anyhow::Result<()> {
    match args.sc {
        None => link_show(LinkShowArgs::default()),
        Some(LinkSubcommand::Show(args)) => link_show(args),
    }
}

fn link_show(show_args: LinkShowArgs) -> anyhow::Result<()> {
    let mut nl = NetlinkRouteHandle::open()?;

    let req = match show_args.ifname {
        None => LinkGetRequest::all(),
        Some(n) => LinkGetRequest::for_ifname(n.as_str())?,
    };

    let response = req.send(&mut nl)?;

    println!("{:#?}", response);

    Ok(())
}
