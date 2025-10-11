use std::net::IpAddr;

use log::error;
use tokio::sync::mpsc;

mod ipc;
mod mdnsresponder_error;

#[derive(Debug)]
pub struct Service
{
    pub name: String,
    pub service_type: String,
    pub domain: String,
}

#[derive(Debug)]
pub struct Resolved
{
    pub full_name: String,
    pub host_target: String,
    pub port: u16,
    pub txt_data: Vec<String>,
}

#[derive(Debug)]
pub struct AddressInfo
{
    pub hostname: String,
    pub address: IpAddr,
}

#[derive(Debug)]
pub enum MDnsResponderEvent
{
    ServiceAdded(Service),
    ServiceRemoved(Service),
    ServiceResolved(Resolved),
    AddressInfoResolved(AddressInfo),
}

#[derive(Debug)]
pub enum Protocol
{
    IPv4,
    IPv6,
    Both,
}

pub struct MDnsResponder
{
    ipc: ipc::Ipc,
    pub events: mpsc::Receiver<MDnsResponderEvent>,
}

impl MDnsResponder
{
    /// Creates a new instance of `MDnsResponder` with the specified channel buffer size.
    ///
    /// # Arguments
    ///
    /// * `channel_buffer_size` - The size of the buffer for the internal channels. Must be greater than zero.
    ///
    /// # Errors
    ///
    /// Returns `Err(MDnsResponderError::ChannelCreationFailed)` if the buffer size is zero.
    /// Returns `Err(MDnsResponderError::IpcConnectionCreationFailed)` if IPC creation fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libmdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let responder = MDnsResponder::new(10).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(
        channel_buffer_size: usize,
    ) -> Result<Self, mdnsresponder_error::MDnsResponderError>
    {
        if channel_buffer_size == 0
        {
            error!("Channel buffer size must be greater than zero");
            return Err(mdnsresponder_error::MDnsResponderError::ChannelCreationFailed);
        }

        let (events_sender, events_receiver) = mpsc::channel(channel_buffer_size);

        let ipc = match ipc::Ipc::new(events_sender).await
        {
            Ok(ipc) => ipc,
            Err(e) =>
            {
                error!("Failed to create IPC: {}", e);
                return Err(mdnsresponder_error::MDnsResponderError::IpcConnectionCreationFailed);
            }
        };

        return Ok(MDnsResponder
        {
            ipc,
            events: events_receiver,
        });
    }

    /// Closes the `MDnsResponder` instance, releasing any associated resources.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libmdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let responder = MDnsResponder::new(10).await?;
    ///     responder.close().await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn close(self)
    {
        self.ipc.close().await;
    }

    /// Starts browsing for services of the specified type and domain.
    ///
    /// # Arguments
    ///
    /// * `service_type` - The type of service to browse for (e.g., "_http._tcp").
    /// * `service_domain` - The domain in which to browse for the service (e.g., "local").
    ///
    /// # Returns
    ///
    /// Returns a unique context identifier for the browse request.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libmdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.browse("_http._tcp".to_string(), "local".to_string()).await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn browse(
        &mut self, service_type: String,
        service_domain: String
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self
            .ipc
            .write_browse_request(service_type, service_domain)
            .await
        {
            Ok(context) => Ok(context),
            Err(e) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }

    /// Starts resolving a service with the specified name, type, and domain.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service to resolve (e.g., "My Service").
    /// * `service_type` - The type of service to resolve (e.g., "_http._tcp").
    /// * `service_domain` - The domain in which to resolve the service (e.g., "local").
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libmdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.resolve("My Service".to_string(), "_http._tcp".to_string(), "local".to_string()).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn resolve(
        &mut self,
        service_name: String,
        service_type: String,
        service_domain: String,
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self
            .ipc
            .write_resolve_request(
                service_name,
                service_type,
                service_domain,
            )
            .await
        {
            Ok(context) => Ok(context),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }

    /// Resolves the given hostname to its corresponding IP addresses, IPv4, IPv6, or both.
    ///
    /// # Arguments
    ///
    /// * `hostname` - The hostname to resolve (e.g., "example.local").
    /// * `protocol` - The protocol to use for resolution (IPv4, IPv6, or Both).
    ///
    /// # Returns
    ///
    /// Returns a unique context identifier for the address info request.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libmdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.get_addr_info("example.local".to_string(), Protocol::Both).await?;
    ///     responder.cancel(context).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_addr_info(&mut self, hostname: String, protocol: Protocol) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write_addrinfo_request(protocol, hostname).await
        {
            Ok(context) => Ok(context),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }

    /// Cancels an ongoing browse or resolve operation identified by the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - The unique context identifier returned by `browse`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use libmdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.browse("_http._tcp".to_string(), "local".to_string()).await?;
    ///     responder.cancel(context).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn cancel(&mut self, context: u64) -> Result<(), mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write_cancel_request(context).await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }
}
