pub const REPLY_OPERATION_START: u32 = 64;

pub enum ReplyOperation
{
    Enumeration,
    RegisterService,
    Browse,
    Resolve,
    Query,
    RegisterRecord,
    GetProperty,
    PortMapping,
    AddressInfo,
    AsyncError,
}

impl ReplyOperation
{
    pub fn from_u32(value: u32) -> Option<ReplyOperation>
    {
        match value
        {
            64 => Some(ReplyOperation::Enumeration),
            65 => Some(ReplyOperation::RegisterService),
            66 => Some(ReplyOperation::Browse),
            67 => Some(ReplyOperation::Resolve),
            68 => Some(ReplyOperation::Query),
            69 => Some(ReplyOperation::RegisterRecord),
            70 => Some(ReplyOperation::GetProperty),
            71 => Some(ReplyOperation::PortMapping),
            72 => Some(ReplyOperation::AddressInfo),
            73 => Some(ReplyOperation::AsyncError),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32
    {
        match self
        {
            ReplyOperation::Enumeration => 64,
            ReplyOperation::RegisterService => 65,
            ReplyOperation::Browse => 66,
            ReplyOperation::Resolve => 67,
            ReplyOperation::Query => 68,
            ReplyOperation::RegisterRecord => 69,
            ReplyOperation::GetProperty => 70,
            ReplyOperation::PortMapping => 71,
            ReplyOperation::AddressInfo => 72,
            ReplyOperation::AsyncError => 73,
        }
    }
}

impl std::fmt::Debug for ReplyOperation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self
        {
            ReplyOperation::Enumeration => write!(f, "Enumeration"),
            ReplyOperation::RegisterService => write!(f, "RegisterService"),
            ReplyOperation::Browse => write!(f, "Browse"),
            ReplyOperation::Resolve => write!(f, "Resolve"),
            ReplyOperation::Query => write!(f, "Query"),
            ReplyOperation::RegisterRecord => write!(f, "RegisterRecord"),
            ReplyOperation::GetProperty => write!(f, "GetProperty"),
            ReplyOperation::PortMapping => write!(f, "PortMapping"),
            ReplyOperation::AddressInfo => write!(f, "AddressInfo"),
            ReplyOperation::AsyncError => write!(f, "AsyncError"),
        }
    }
}
