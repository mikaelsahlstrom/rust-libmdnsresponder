use libmdnsresponder;
use log::{debug, info};

#[tokio::main]
async fn main()
{
    env_logger::init();

    debug!("Starting service discovery");

    let mut responder = libmdnsresponder::MDnsResponder::new(10).await.unwrap();

    let browse_context = responder.browse("_sonos._tcp", "local").await;

    debug!("Service discovery started, waiting for services or Ctrl+C to exit");

    loop
    {
        tokio::select!
        {
            Some(event) = responder.events.recv() =>
            {
                match event
                {
                    libmdnsresponder::MDnsResponderEvent::ServiceAdded(service) =>
                    {
                        info!("Service Added: {:?}", service);

                        // Resolve the service to get more details.
                        let resolve_context = responder.resolve(&service.name, &service.service_type, &service.domain).await;
                    }
                    libmdnsresponder::MDnsResponderEvent::ServiceRemoved(service) =>
                    {
                        info!("Service Removed: {:?}", service);
                    }
                    libmdnsresponder::MDnsResponderEvent::ServiceResolved(resolved) =>
                    {
                        info!("Service Resolved: {:?}", resolved);
                    }
                }
            }
            _ = tokio::signal::ctrl_c() =>
            {
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
