pub enum RequestOperation
{
    None,
    Connection,
    RegisterRecord,
    RemoveRecord,
    Enumeration,
    RegisterService,
    Browse,
    Resolve,
    Query,
    ReconfirmRecord,
    AddRecord,
    UpdateRecord,
    SetDomain,
    GetProperty,
    PortMapping,
    AddressInfo,
    SendBpfObsolete,
    GetPid,
    Release,
    ConnectionDelegate,
    Cancel,
}

impl RequestOperation
{
    pub fn from_u32(value: u32) -> Option<RequestOperation>
    {
        return match value
        {
            0 => Some(RequestOperation::None),
            1 => Some(RequestOperation::Connection),
            2 => Some(RequestOperation::RegisterRecord),
            3 => Some(RequestOperation::RemoveRecord),
            4 => Some(RequestOperation::Enumeration),
            5 => Some(RequestOperation::RegisterService),
            6 => Some(RequestOperation::Browse),
            7 => Some(RequestOperation::Resolve),
            8 => Some(RequestOperation::Query),
            9 => Some(RequestOperation::ReconfirmRecord),
            10 => Some(RequestOperation::AddRecord),
            11 => Some(RequestOperation::UpdateRecord),
            12 => Some(RequestOperation::SetDomain),
            13 => Some(RequestOperation::GetProperty),
            14 => Some(RequestOperation::PortMapping),
            15 => Some(RequestOperation::AddressInfo),
            16 => Some(RequestOperation::SendBpfObsolete),
            17 => Some(RequestOperation::GetPid),
            18 => Some(RequestOperation::Release),
            19 => Some(RequestOperation::ConnectionDelegate),
            63 => Some(RequestOperation::Cancel),
            _ => None,
        };
    }

    pub fn to_u32(&self) -> u32
    {
        return match self
        {
            RequestOperation::None => 0,
            RequestOperation::Connection => 1,
            RequestOperation::RegisterRecord => 2,
            RequestOperation::RemoveRecord => 3,
            RequestOperation::Enumeration => 4,
            RequestOperation::RegisterService => 5,
            RequestOperation::Browse => 6,
            RequestOperation::Resolve => 7,
            RequestOperation::Query => 8,
            RequestOperation::ReconfirmRecord => 9,
            RequestOperation::AddRecord => 10,
            RequestOperation::UpdateRecord => 11,
            RequestOperation::SetDomain => 12,
            RequestOperation::GetProperty => 13,
            RequestOperation::PortMapping => 14,
            RequestOperation::AddressInfo => 15,
            RequestOperation::SendBpfObsolete => 16,
            RequestOperation::GetPid => 17,
            RequestOperation::Release => 18,
            RequestOperation::ConnectionDelegate => 19,
            RequestOperation::Cancel => 63,
        };
    }
}

impl std::fmt::Debug for RequestOperation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        return write!(
            f,
            "{:?}",
            match self
            {
                RequestOperation::None => "None",
                RequestOperation::Connection => "Connection",
                RequestOperation::RegisterRecord => "RegisterRecord",
                RequestOperation::RemoveRecord => "RemoveRecord",
                RequestOperation::Enumeration => "Enumeration",
                RequestOperation::RegisterService => "RegisterService",
                RequestOperation::Browse => "Browse",
                RequestOperation::Resolve => "Resolve",
                RequestOperation::Query => "Query",
                RequestOperation::ReconfirmRecord => "ReconfirmRecord",
                RequestOperation::AddRecord => "AddRecord",
                RequestOperation::UpdateRecord => "UpdateRecord",
                RequestOperation::SetDomain => "SetDomain",
                RequestOperation::GetProperty => "GetProperty",
                RequestOperation::PortMapping => "PortMapping",
                RequestOperation::AddressInfo => "AddressInfo",
                RequestOperation::SendBpfObsolete => "SendBpfObsolete",
                RequestOperation::GetPid => "GetPid",
                RequestOperation::Release => "Release",
                RequestOperation::ConnectionDelegate => "ConnectionDelegate",
                RequestOperation::Cancel => "Cancel",
            }
        );
    }
}
