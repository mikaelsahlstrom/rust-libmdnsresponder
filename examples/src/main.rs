use log::{ debug, info };
use libmdnsresponder;

#[tokio::main]
async fn main()
{
    env_logger::init();

    debug!("Starting service discovery");

    let mut responder = libmdnsresponder::MDnsResponder::new(10).await.unwrap();

    let browse_context = responder.browse("_sonos._tcp", "local").await;

    debug!("Service discovery started, waiting for services or Ctrl+C to exit");

    loop {
        tokio::select! {
            Some(service) = responder.service_added.recv() => {
                info!("Service added: {}", service);
            }
            Some(service) = responder.service_removed.recv() => {
                info!("Service removed: {}", service);
            }
            _ = tokio::signal::ctrl_c() => {
                debug!("Ctrl+C received, stopping service discovery");
                break;
            }
        }
    }

    // Cancel the browse operation.
    responder.cancel(browse_context).await;

    // Close down the responder.
    responder.close().await;

    debug!("Service discovery stopped");
}
