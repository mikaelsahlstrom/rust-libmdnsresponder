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

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Protocol
{
    IPv4 = 0x1,
    IPv6 = 0x2,
    Both = 0x3,
}

pub struct Request
{
    service_flags: ServiceFlags,
    interface_index: u32,
    protocol: Protocol,
    hostname: String,
}

pub struct Reply
{
    pub header: super::ReplyHeader,
    pub name: String,
    pub rrtype: u16,
    pub rrclass: u16,
    pub rdlen: u16,
    pub rdata: Vec<u8>,
    pub ttl: u32,
}

impl Request
{
    pub fn new(
        service_flags: ServiceFlags,
        interface_index: u32,
        protocol: Protocol,
        hostname: String,
    ) -> Self
    {
        return Request
        {
            service_flags,
            interface_index,
            protocol,
            hostname,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();

        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());
        buf.extend_from_slice(&self.interface_index.to_be_bytes());
        buf.extend_from_slice(&(self.protocol as u32).to_be_bytes());
        buf.extend_from_slice(self.hostname.as_bytes());
        buf.push(0); // Null-terminate the hostname

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
        let header = super::ReplyHeader::from_bytes(&buf[0..12])?;

        let mut offset = 12;

        let name = Self::cstr_from_buf(&buf[offset..]);
        offset += name.len() + 1;

        if buf.len() < offset + 6
        {
            return Err(format!("Buffer too short for RR fields: {}", buf.len()));
        }

        let rrtype = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        let rrclass = u16::from_be_bytes([buf[offset + 2], buf[offset + 3]]);
        let rdlen = u16::from_be_bytes([buf[offset + 4], buf[offset + 5]]);
        offset += 6;

        if buf.len() < offset + (rdlen as usize) + 4
        {
            return Err(format!("Buffer too short for RDATA and TTL: {}", buf.len()));
        }

        let rdata = buf[offset..offset + (rdlen as usize)].to_vec();
        offset += rdlen as usize;

        let ttl = u32::from_be_bytes([
            buf[offset],
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
        ]);

        return Ok(Reply
        {
            header,
            name,
            rrtype,
            rrclass,
            rdlen,
            rdata,
            ttl,
        });
    }
}

impl From<crate::Protocol> for Protocol
{
    fn from(proto: crate::Protocol) -> Self
    {
        match proto
        {
            crate::Protocol::IPv4 => Protocol::IPv4,
            crate::Protocol::IPv6 => Protocol::IPv6,
            crate::Protocol::Both => Protocol::Both,
        }
    }
}
