#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ServiceFlags
{
    none = 0x0,
    auto_trigger = 0x1,
    add = 0x2,
    default = 0x3,
    force_multicast = 0x400,
    include_p2p = 0x20000,
    include_awdl = 0x100000
}

pub struct Request
{
    service_flags: ServiceFlags,
    interface_index: u32,
    reg_type: String,
    domain: String,
}

#[derive(Debug, PartialEq)]
pub enum ReplyFlags
{
    more_coming = 0x1,
    add = 0x2,
    threshold_reached = 0x2000000
}

#[derive(Debug)]
pub struct ReplyHeader
{
    flags: Vec<ReplyFlags>,
    interface_index: u32,
    error: u32
}

#[derive(Debug)]
pub struct Reply
{
    pub header: ReplyHeader,
    pub service_name: String,
    pub service_type: String,
    pub service_domain: String
}


impl Request
{
    pub fn new(service_flags: ServiceFlags, interface_index: u32, reg_type: String, domain: String) -> Self
    {
        Request { service_flags, interface_index, reg_type, domain }
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());
        buf.extend_from_slice(&self.interface_index.to_be_bytes());
        buf.extend_from_slice(self.reg_type.as_bytes());
        buf.push(0); // Null terminator for string
        buf.extend_from_slice(self.domain.as_bytes());
        buf.push(0); // Null terminator for string
        return buf;
    }
}

impl ReplyFlags
{
    pub fn from_u32(value: u32) -> Result<Vec<Self>, String>
    {
        let mut flags = Vec::new();

        if value & (ReplyFlags::more_coming as u32) != 0 {
            flags.push(ReplyFlags::more_coming);
        }
        if value & (ReplyFlags::add as u32) != 0 {
            flags.push(ReplyFlags::add);
        }
        if value & (ReplyFlags::threshold_reached as u32) != 0 {
            flags.push(ReplyFlags::threshold_reached);
        }
        if flags.is_empty() {
            Err(format!("Unknown ReplyFlags value: {}", value))
        } else {
            Ok(flags)
        }
    }
}

impl ReplyHeader
{
    pub fn from_bytes(buf: &[u8]) -> Result<Self, String>
    {
        if buf.len() < 12
        {
            return Err(format!("Buffer too short for ReplyHeader: {}", buf.len()));
        }

        let flags = ReplyFlags::from_u32(u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]))?;
        let interface_index = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let error = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

        Ok(ReplyHeader { flags, interface_index, error })
    }
}

impl Reply
{
    fn cstr_from_buf(buf: &[u8]) -> String
    {
        let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        return String::from_utf8_lossy(&buf[..nul_pos]).into_owned();
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, String>
    {
        let header = ReplyHeader::from_bytes(&buf)?;

        let mut pos = 12;

        let service_name = Self::cstr_from_buf(&buf[pos..]);
        pos += service_name.len() + 1;

        let service_type = Self::cstr_from_buf(&buf[pos..]);
        pos += service_type.len() + 1;

        let service_domain = Self::cstr_from_buf(&buf[pos..]);

        Ok(Reply { header, service_name, service_type, service_domain })
    }

    pub fn is_add(&self) -> bool
    {
        return self.header.flags.contains(&ReplyFlags::add);
    }
}
