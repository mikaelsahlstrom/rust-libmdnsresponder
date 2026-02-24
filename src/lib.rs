use std::net::IpAddr;

use log::error;
use tokio::sync::mpsc;

mod ipc;
mod mdnsresponder_error;

pub use mdnsresponder_error::MDnsResponderError;

#[derive(Debug)]
pub struct Service
{
    pub interface_index: u32,
    pub name: String,
    pub service_type: String,
    pub domain: String,
}

#[derive(Debug)]
pub struct Resolved
{
    pub interface_index: u32,
    pub full_name: String,
    pub host_target: String,
    pub port: u16,
    pub txt_data: Vec<String>,
}

#[derive(Debug)]
pub struct AddressInfo
{
    pub interface_index: u32,
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
    /// use mdnsresponder::MDnsResponder;
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
    /// use mdnsresponder::MDnsResponder;
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
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.browse(0, "_http._tcp".to_string(), "local".to_string()).await;
    ///     Ok(())
    /// }
    /// ```
    pub async fn browse(
        &mut self,
        interface_index: u32,
        service_type: String,
        service_domain: String
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self
            .ipc
            .write
            .browse_request(interface_index, service_type, service_domain)
            .await
        {
            Ok(context) => Ok(context),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
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
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.resolve(0, "My Service".to_string(), "_http._tcp".to_string(), "local".to_string()).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn resolve(
        &mut self,
        interface_index: u32,
        service_name: String,
        service_type: String,
        service_domain: String,
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self
            .ipc
            .write
            .resolve_request(
                interface_index,
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
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.get_addr_info(0, "example.local".to_string(), mdnsresponder::Protocol::Both).await?;
    ///     responder.cancel(context).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_addr_info(
        &mut self,
        interface_index: u32,
        hostname: String,
        protocol: Protocol
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write.addrinfo_request(interface_index, protocol, hostname).await
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
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.browse(0, "_http._tcp".to_string(), "local".to_string()).await?;
    ///     responder.cancel(context).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn cancel(&mut self, context: u64) -> Result<(), mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write.cancel_request(context).await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }

    /// Registers a service with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `interface_index` - The index of the network interface to use for registration, 0 for all interfaces.
    /// * `name` - The name of the service to register (e.g., "My Service").
    /// * `service_type` - The type of service to register (e.g., "_http._tcp").
    /// * `domain` - The domain in which to register the service (e.g., "local").
    /// * `host` - The hostname of the service (e.g., "myhost.local"), empty string for this host.
    /// * `port` - The port number on which the service is available.
    /// * `txt_data` - A vector of strings representing the TXT records associated with the service.
    ///
    /// # Returns
    ///
    /// Returns a unique context identifier for the registration request.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.register(0, "My Service".to_string(), "_http._tcp".to_string(), "local".to_string(), "myhost.local".to_string(), 8080, vec!["key=value".to_string()]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn register(
        &mut self,
        interface_index: u32,
        name: String,
        service_type: String,
        domain: String,
        host: String,
        port: u16,
        txt_data: Vec<String>
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write.register_request(interface_index, name, service_type, domain, host, port, txt_data).await
        {
            Ok(context) => Ok(context),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }

    /// Adds a DNS resource record with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `context` - The unique context identifier returned by `register`.
    /// * `rrtype` - The resource record type (e.g., 1 for A, 28 for AAAA).
    /// * `rdata` - The raw resource record data as a byte vector.
    /// * `ttl` - The time to live value in seconds.
    ///
    /// # Returns
    ///
    /// Returns a result indicating the success or failure of the add record request.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.register(0, "My Service".to_string(), "_http._tcp".to_string(), "local".to_string(), "myhost.local".to_string(), 8080, vec!["key=value".to_string()]).await?;
    ///     responder.add_record(context, 1, vec![192, 168, 1, 1], 4500).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_record(
        &mut self,
        context: u64,
        rrtype: u16,
        rdata: Vec<u8>,
        ttl: u32
    ) -> Result<(), mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write.add_record_request(context, rrtype, rdata, ttl).await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }

    /// Registers an individual DNS resource record with the specified name and parameters.
    ///
    /// Unlike `add_record`, which attaches an extra record to a service registered via
    /// `register`, this method registers a standalone record with an arbitrary fully-qualified
    /// domain name. Use this to publish A/AAAA records for a hostname, for example.
    ///
    /// # Arguments
    ///
    /// * `interface_index` - The network interface index (0 for all interfaces).
    /// * `fullname` - The full DNS name for the record (e.g., "myhost.local").
    /// * `rrtype` - The resource record type (e.g., 1 for A, 28 for AAAA).
    /// * `rrclass` - The resource record class (e.g., 1 for IN).
    /// * `rdata` - The raw resource record data as a byte vector.
    /// * `ttl` - The time to live value in seconds.
    ///
    /// # Returns
    ///
    /// Returns a unique context identifier that can be passed to `cancel`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use mdnsresponder::MDnsResponder;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut responder = MDnsResponder::new(10).await?;
    ///     let context = responder.register_record(0, "myhost.local".to_string(), 1, 1, vec![192, 168, 1, 1], 4500).await?;
    ///     responder.cancel(context).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn register_record(
        &mut self,
        interface_index: u32,
        fullname: String,
        rrtype: u16,
        rrclass: u16,
        rdata: Vec<u8>,
        ttl: u32
    ) -> Result<u64, mdnsresponder_error::MDnsResponderError>
    {
        return match self.ipc.write.register_record_request(interface_index, fullname, rrtype, rrclass, rdata, ttl).await
        {
            Ok(context) => Ok(context),
            Err(_) => Err(mdnsresponder_error::MDnsResponderError::IpcWriteFailed),
        };
    }
}
