use tokio::sync::mpsc;
use log::{ error };

mod ipc;
mod mdnsresponder_error;

pub struct MDnsResponder
{
    ipc: ipc::Ipc,
    pub service_added: mpsc::Receiver<String>,
    pub service_removed: mpsc::Receiver<String>,
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
    /// ```rust
    /// let responder = MDnsResponder::new(10).await?;
    /// ```
    pub async fn new(channel_buffer_size: usize) -> Result<Self, mdnsresponder_error::MDnsResponderError>
    {
        if channel_buffer_size == 0
        {
            error!("Channel buffer size must be greater than zero");
            return Err(mdnsresponder_error::MDnsResponderError::ChannelCreationFailed);
        }

        let (service_added_sender, service_added) = mpsc::channel(channel_buffer_size);
        let (service_removed_sender, service_removed) = mpsc::channel(channel_buffer_size);

        let ipc = match ipc::Ipc::new(service_added_sender, service_removed_sender).await
        {
            Ok(ipc) => ipc,
            Err(e) => {
                error!("Failed to create IPC: {}", e);
                return Err(mdnsresponder_error::MDnsResponderError::IpcConnectionCreationFailed);
            }
        };

        return Ok(MDnsResponder { ipc, service_added, service_removed });
    }

    /// Closes the `MDnsResponder` instance, releasing any associated resources.
    ///
    /// # Examples
    ///
    /// ```rust
    /// responder.close().await;
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
    /// ```rust
    /// let context = responder.browse("_http._tcp", "local").await;
    /// ```
    pub async fn browse(&mut self, service_type: &str, service_domain: &str) -> u64
    {
        return self.ipc.write_browse_request(service_type.to_string(), service_domain.to_string()).await;
    }

    /// Cancels a previously started browse operation identified by the given context.
    ///
    /// # Arguments
    ///
    /// * `context` - The unique context identifier returned by `browse`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// responder.cancel(context).await;
    /// ```
    pub async fn cancel(&mut self, context: u64)
    {
        self.ipc.write_cancel_request(context).await;
    }
}
