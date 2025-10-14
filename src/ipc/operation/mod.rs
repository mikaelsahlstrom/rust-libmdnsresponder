pub mod browse;
pub mod resolve;
pub mod addrinfo;
pub mod publish;

#[derive(Debug, PartialEq, Eq)]
pub enum ReplyFlags
{
    MoreComing = 0x1,
    Add = 0x2,
    ThresholdReached = 0x2000000,
}

#[derive(Debug)]
pub struct ReplyHeader
{
    flags: Vec<ReplyFlags>,
    interface_index: u32,
    error: u32,
}

impl ReplyFlags
{
    pub fn from_u32(value: u32) -> Vec<Self>
    {
        let mut flags = Vec::new();

        if value & (ReplyFlags::MoreComing as u32) != 0
        {
            flags.push(ReplyFlags::MoreComing);
        }

        if value & (ReplyFlags::Add as u32) != 0
        {
            flags.push(ReplyFlags::Add);
        }

        if value & (ReplyFlags::ThresholdReached as u32) != 0
        {
            flags.push(ReplyFlags::ThresholdReached);
        }

        return flags;
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

        let flags = ReplyFlags::from_u32(u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]));
        let interface_index = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let error = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

        return Ok(ReplyHeader
        {
            flags,
            interface_index,
            error,
        });
    }
}
