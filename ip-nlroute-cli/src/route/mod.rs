use clap::{Args, Subcommand};
use ip_nlroute::NetlinkRouteHandle;
use ip_nlroute::route::get::RouteGetRequest;

#[derive(Args, Debug)]
pub struct RouteArgs {
    #[clap(subcommand)]
    sc: Option<RouteSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum RouteSubcommand {
    Show,
}

pub fn route_main(args: RouteArgs) -> anyhow::Result<()> {
    match args.sc {
        None | Some(RouteSubcommand::Show) => route_show(),
    }
}

fn route_show() -> anyhow::Result<()> {
    let mut nl = NetlinkRouteHandle::open()?;
    let req = RouteGetRequest::new();
    let response = req.send(&mut nl)?;

    println!("{:#?}", response.routes);

    Ok(())
}
