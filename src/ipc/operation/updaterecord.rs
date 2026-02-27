use super::ServiceFlags;

pub struct Request
{
    service_flags: ServiceFlags,
    rdata: Vec<u8>,
    ttl: u32,
}

impl Request
{
    pub fn new(
        service_flags: ServiceFlags,
        rdata: Vec<u8>,
        ttl: u32,
    ) -> Self
    {
        return Request
        {
            service_flags,
            rdata,
            ttl,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();

        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());
        buf.extend_from_slice(&(self.rdata.len() as u16).to_be_bytes());
        buf.extend_from_slice(&self.rdata);
        buf.extend_from_slice(&self.ttl.to_be_bytes());

        return buf;
    }
}
