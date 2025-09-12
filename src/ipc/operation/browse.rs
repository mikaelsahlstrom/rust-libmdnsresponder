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

        if offset >= buf.len() {
            return Err("Buffer too short to contain service name".to_string());
        }

        let service_name = Self::cstr_from_buf(&buf[offset..]);
        offset += service_name.len() + 1;

        if offset >= buf.len() {
            return Err("Buffer too short to contain service type".to_string());
        }

        let service_type = Self::cstr_from_buf(&buf[offset..]);
        offset += service_type.len() + 1;

        if offset >= buf.len() {
            return Err("Buffer too short to contain service domain".to_string());
        }

        let service_domain = Self::cstr_from_buf(&buf[offset..]);

        Ok(Reply { header, service_name, service_type, service_domain })
    }
}
