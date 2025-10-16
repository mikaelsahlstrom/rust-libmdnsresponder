use super::ServiceFlags;

pub struct Request
{
    service_flags: ServiceFlags,
    interface_index: u32,
    name: String,
    service_type: String,
    domain: String,
    host: String,
    port: u16,
    txt_data: Vec<String>,
}

pub struct Reply
{
    pub header: super::ReplyHeader,
    // No reply data for register operation
}

impl Request
{
    pub fn new(
        service_flags: ServiceFlags,
        interface_index: u32,
        name: String,
        service_type: String,
        domain: String,
        host: String,
        port: u16,
        txt_data: Vec<String>,
    ) -> Self
    {
        return Request
        {
            service_flags,
            interface_index,
            name,
            service_type,
            domain,
            host,
            port,
            txt_data,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();

        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());
        buf.extend_from_slice(&self.interface_index.to_be_bytes());

        buf.extend_from_slice(self.name.as_bytes());
        buf.push(0);

        buf.extend_from_slice(self.service_type.as_bytes());
        buf.push(0);

        buf.extend_from_slice(self.domain.as_bytes());
        buf.push(0);

        buf.extend_from_slice(self.host.as_bytes());
        buf.push(0);

        buf.extend_from_slice(&self.port.to_be_bytes());

        let txt_len: u16 = self.txt_data.iter().map(|s| s.len() as u16 + 1).sum();
        buf.extend_from_slice(&txt_len.to_be_bytes());

        for txt in &self.txt_data
        {
            buf.push(txt.len() as u8);
            buf.extend_from_slice(txt.as_bytes());
        }

        return buf;
    }
}

impl Reply
{
    pub fn from_bytes(buf: &[u8]) -> Result<Self, String>
    {
        let header = super::ReplyHeader::from_bytes(&buf[0..12])?;

        return Ok(Reply
        {
            header,
        });
    }
}
