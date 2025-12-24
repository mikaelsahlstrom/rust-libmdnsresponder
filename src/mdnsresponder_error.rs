// Internal errors that are used only within the library and do not reach users
#[derive(Debug)]
pub(crate) enum InternalError
{
    IncompleteFrame,
    FrameParsingFailed,
    MDnsResponderError(MDnsResponderErrorCode),
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
            InternalError::MDnsResponderError(code) =>
            {
                write!(f, "mDNS Responder error: {:?}", code)
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
    MDnsResponderError(MDnsResponderErrorCode),
}

#[derive(Debug)]
#[repr(i32)]
pub enum MDnsResponderErrorCode
{
    NoError = 0,
    Unknown = -65537,
    NoSuchName = -65538,
    NoMemory = -65539,
    BadParam = -65540,
    BadReference = -65541,
    BadState = -65542,
    BadFlags = -65543,
    Unsupported = -65544,
    AlreadyRegistered = -65547,
    NameConflict = -65548,
    Invalid = -65549,
    NoSuchRecord = -65554,
    PolicyDenied = -65567,
    Other(i32),
}

impl MDnsResponderErrorCode
{
    pub fn from_i32(value: i32) -> Self
    {
        match value
        {
            0 => MDnsResponderErrorCode::NoError,
            -65537 => MDnsResponderErrorCode::Unknown,
            -65538 => MDnsResponderErrorCode::NoSuchName,
            -65539 => MDnsResponderErrorCode::NoMemory,
            -65540 => MDnsResponderErrorCode::BadParam,
            -65541 => MDnsResponderErrorCode::BadReference,
            -65542 => MDnsResponderErrorCode::BadState,
            -65543 => MDnsResponderErrorCode::BadFlags,
            -65544 => MDnsResponderErrorCode::Unsupported,
            -65547 => MDnsResponderErrorCode::AlreadyRegistered,
            -65548 => MDnsResponderErrorCode::NameConflict,
            -65549 => MDnsResponderErrorCode::Invalid,
            -65554 => MDnsResponderErrorCode::NoSuchRecord,
            -65567 => MDnsResponderErrorCode::PolicyDenied,
            other => MDnsResponderErrorCode::Other(other),
        }
    }
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
            MDnsResponderError::MDnsResponderError(code) =>
            {
                write!(f, "mDNS Responder error: {:?}", code)
            }
        }
    }
}

impl std::error::Error for MDnsResponderError {}
