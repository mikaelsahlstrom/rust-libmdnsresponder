use super::ServiceFlags;

pub struct Request
{
    service_flags: ServiceFlags,
}

impl Request
{
    pub fn new(
        service_flags: ServiceFlags,
    ) -> Self
    {
        return Request
        {
            service_flags,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();

        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());

        return buf;
    }
}
