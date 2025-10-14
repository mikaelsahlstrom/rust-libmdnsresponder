// Internal errors that are used only within the library and do not reach users
#[derive(Debug)]
pub(crate) enum InternalError
{
    IncompleteFrame,
    FrameParsingFailed,
}

impl std::fmt::Display for InternalError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            InternalError::IncompleteFrame =>
            {
                write!(f, "Incomplete frame received")
            }
            InternalError::FrameParsingFailed =>
            {
                write!(f, "Failed to parse frame")
            }
        }
    }
}

impl std::error::Error for InternalError {}

// Public errors that can reach users of the library
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
