# rust-libmdnsresponder

A rust library that provides an interface to the Apple mDNSResponder API over the unix socket interface mDNSResponder provides.

# Usage example

Example of browsing for HTTP services on the local network and resolving them to get more details, including their IP addresses.

Browsing for services returns a Service:

```rust
pub struct Service
{
    pub interface_index: u32,
    pub name: String,
    pub service_type: String,
    pub domain: String,
}
````

Resolving a service returns a Resolved:

```rust
pub struct Resolved
{
    pub interface_index: u32,
    pub full_name: String,
    pub host_target: String,
    pub port: u16,
    pub txt_data: Vec<String>,
}
````

Getting address info returns an AddressInfo:

```rust
pub struct AddressInfo
{
    pub interface_index: u32,
    pub hostname: String,
    pub address: IpAddr,
}
```

Full example:

```rust
use mdnsresponder::{
    MDnsResponder,
    MDnsResponderEvent,
    Protocol
};

#[tokio::main]
async fn main()
{
    env_logger::init();

    let mut responder = MDnsResponder::new(10).await.unwrap();

    let browse_context = match responder.browse(
        0,
        "_http._tcp".to_string(),
        "local".to_string()).await
    {
        Ok(context) => context,
        Err(e) =>
        {
            error!("Failed to start browsing: {:?}", e);
            return;
        }
    };


    loop
    {
        tokio::select!
        {
            Some(event) = responder.events.recv() =>
            {
                match event
                {
                    MDnsResponderEvent::ServiceAdded(service) =>
                    {
                        info!("Service Added: {:?}", service);

                        // Resolve the service to get more details.
                        let _resolve_context = responder.resolve(service.interface_index, service.name, service.service_type, service.domain).await;
                    }
                    MDnsResponderEvent::ServiceRemoved(service) =>
                    {
                        info!("Service Removed: {:?}", service);
                    }
                    MDnsResponderEvent::ServiceResolved(resolved) =>
                    {
                        info!("Service Resolved: {:?}", resolved);

                        let _addr_info_context = responder.get_addr_info(resolved.interface_index, resolved.host_target, Protocol::Both).await;
                    }
                    MDnsResponderEvent::AddressInfoResolved(addr_info) =>
                    {
                        info!("Address Info Resolved: {:?}", addr_info);
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
    match responder.cancel(browse_context).await
    {
        Ok(_) => debug!("Browse operation cancelled"),
        Err(e) => error!("Failed to cancel browse operation: {:?}", e),
    }

    // Close down the responder.
    responder.close().await;
}
```
