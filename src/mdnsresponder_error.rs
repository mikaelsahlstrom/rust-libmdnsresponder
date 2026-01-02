// Internal errors that are used only within the library and do not reach users
#[derive(Debug)]
pub(crate) enum InternalError
{
    IncompleteFrame,
    FrameParsingFailed,
    MDnsResponderError((ErrorCode, usize)),  // Error code and length of the frame
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
    MDnsResponderError(ErrorCode),
}

#[derive(Debug)]
#[repr(i32)]
pub enum ErrorCode
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

impl ErrorCode
{
    pub fn from_i32(value: i32) -> Self
    {
        match value
        {
            0 => ErrorCode::NoError,
            -65537 => ErrorCode::Unknown,
            -65538 => ErrorCode::NoSuchName,
            -65539 => ErrorCode::NoMemory,
            -65540 => ErrorCode::BadParam,
            -65541 => ErrorCode::BadReference,
            -65542 => ErrorCode::BadState,
            -65543 => ErrorCode::BadFlags,
            -65544 => ErrorCode::Unsupported,
            -65547 => ErrorCode::AlreadyRegistered,
            -65548 => ErrorCode::NameConflict,
            -65549 => ErrorCode::Invalid,
            -65554 => ErrorCode::NoSuchRecord,
            -65567 => ErrorCode::PolicyDenied,
            other => ErrorCode::Other(other),
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
