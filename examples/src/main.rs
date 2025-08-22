use log::{ debug };
use libmdnsresponder;

#[tokio::main]
async fn main()
{
    env_logger::init();

    debug!("Starting service discovery");

    let mut responder = libmdnsresponder::MDnsResponder::new(10).await.unwrap();

    responder.browse("_sonos._tcp", "local").await;

    debug!("Service discovery started, waiting for Ctrl+C to exit");
    tokio::signal::ctrl_c().await.unwrap();
    debug!("Ctrl+C received, stopping service discovery");
    responder.close().await;
    debug!("Service discovery stopped");
}
