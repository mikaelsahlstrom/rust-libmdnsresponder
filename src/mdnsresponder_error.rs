#[derive(Debug)]
pub enum MDnsResponderError
{
    ChannelCreationFailed,
    IpcConnectionCreationFailed,
    IpcWriteFailed,
}

impl std::fmt::Display for MDnsResponderError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            MDnsResponderError::ChannelCreationFailed =>
            {
                write!(f, "Failed to create channel")
            }
            MDnsResponderError::IpcConnectionCreationFailed =>
            {
                write!(f, "Failed to create IPC connection")
            }
            MDnsResponderError::IpcWriteFailed =>
            {
                write!(f, "Failed to write to IPC")
            }
        }
    }
}

impl std::error::Error for MDnsResponderError {}
