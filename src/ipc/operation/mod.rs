pub mod browse;
pub mod resolve;

#[derive(Debug)]
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
