use tokio::sync::mpsc;
use log::{ error };

mod ipc;
mod mdnsresponder_error;

pub struct MDnsResponder
{
    ipc: ipc::Ipc,
    sender: mpsc::Sender<String>,
    pub service_added: mpsc::Receiver<String>,
    pub service_removed: mpsc::Receiver<String>,
}

impl MDnsResponder
{
    pub async fn new(channel_buffer_size: usize) -> Result<Self, mdnsresponder_error::MDnsResponderError>
    {
        if channel_buffer_size == 0
        {
            error!("Channel buffer size must be greater than zero");
            return Err(mdnsresponder_error::MDnsResponderError::ChannelCreationFailed);
        }

        let (sender, service_added) = mpsc::channel(channel_buffer_size);
        let (_, service_removed) = mpsc::channel(channel_buffer_size);

        let ipc = match ipc::Ipc::new().await
        {
            Ok(ipc) => ipc,
            Err(e) => {
                error!("Failed to create IPC: {}", e);
                return Err(mdnsresponder_error::MDnsResponderError::ChannelCreationFailed);
            }
        };

        return Ok(MDnsResponder { ipc, sender, service_added, service_removed });
    }

    pub async fn close(self)
    {
        self.ipc.close().await;
    }

    pub async fn browse(&mut self, service_type: &str, service_domain: &str)
    {
        self.ipc.write_browse_request(service_type.to_string(), service_domain.to_string()).await;
    }
}
