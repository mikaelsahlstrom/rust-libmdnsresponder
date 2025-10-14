#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum ServiceFlags
{
    None = 0x0,
    AutoTrigger = 0x1,
    Add = 0x2,
    Default = 0x3,
    ForceMulticast = 0x400,
    IncludeP2p = 0x20000,
    IncludeAwdl = 0x100000,
}

pub struct Request
{
    service_flags: ServiceFlags,
    interface_index: u32,
    reg_type: String,
    domain: String,
}

#[derive(Debug)]
pub struct Reply
{
    pub header: super::ReplyHeader,
    pub service_name: String,
    pub service_type: String,
    pub service_domain: String,
}

impl Request
{
    pub fn new(
        service_flags: ServiceFlags,
        interface_index: u32,
        reg_type: String,
        domain: String,
    ) -> Self
    {
        return Request
        {
            service_flags,
            interface_index,
            reg_type,
            domain,
        };
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

impl Reply
{
    fn cstr_from_buf(buf: &[u8]) -> String
    {
        let nul_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        return String::from_utf8_lossy(&buf[..nul_pos]).into_owned();
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, String>
    {
        let header = super::ReplyHeader::from_bytes(&buf)?;

        let mut offset = 12;

        if offset >= buf.len()
        {
            return Err("Buffer too short to contain service name".to_string());
        }

        let service_name = Self::cstr_from_buf(&buf[offset..]);
        offset += service_name.len() + 1;

        if offset >= buf.len()
        {
            return Err("Buffer too short to contain service type".to_string());
        }

        let service_type = Self::cstr_from_buf(&buf[offset..]);
        offset += service_type.len() + 1;

        if offset >= buf.len()
        {
            return Err("Buffer too short to contain service domain".to_string());
        }

        let service_domain = Self::cstr_from_buf(&buf[offset..]);

        return Ok(Reply
        {
            header,
            service_name,
            service_type,
            service_domain,
        });
    }

    pub fn is_add(&self) -> bool
    {
        return self.header.flags.contains(&super::ReplyFlags::Add);
    }
}
