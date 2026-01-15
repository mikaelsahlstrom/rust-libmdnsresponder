use super::ServiceFlags;

pub struct Request
{
    service_flags: ServiceFlags,
    interface_index: u32,
    name: String,
    rrtype: u16,
    rrclass: u16,
    rdata: Vec<u8>,
    ttl: u32,
}

impl Request
{
    pub fn new(
        service_flags: ServiceFlags,
        interface_index: u32,
        name: String,
        rrtype: u16,
        rrclass: u16,
        rdata: Vec<u8>,
        ttl: u32,
    ) -> Self
    {
        return Request
        {
            service_flags,
            interface_index,
            name,
            rrtype,
            rrclass,
            rdata,
            ttl,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();

        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());
        buf.extend_from_slice(&self.interface_index.to_be_bytes());
        buf.extend_from_slice(self.name.as_bytes());
        buf.push(0); // Null-terminate the name
        buf.extend_from_slice(&self.rrtype.to_be_bytes());
        buf.extend_from_slice(&self.rrclass.to_be_bytes());
        buf.extend_from_slice(&(self.rdata.len() as u16).to_be_bytes());
        buf.extend_from_slice(&self.rdata);
        buf.extend_from_slice(&self.ttl.to_be_bytes());

        return buf;
    }
}
