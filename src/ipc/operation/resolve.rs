const ESCAPED_BYTE_SMALL: &str = 
    "\\000\\001\\002\\003\\004\\005\\006\\007\\008\\009\
     \\010\\011\\012\\013\\014\\015\\016\\017\\018\\019\
     \\020\\021\\022\\023\\024\\025\\026\\027\\028\\029\
     \\030\\031";

const ESCAPED_BYTE_LARGE: &str = 
    "\\127\\128\\129\
     \\130\\131\\132\\133\\134\\135\\136\\137\\138\\139\
     \\140\\141\\142\\143\\144\\145\\146\\147\\148\\149\
     \\150\\151\\152\\153\\154\\155\\156\\157\\158\\159\
     \\160\\161\\162\\163\\164\\165\\166\\167\\168\\169\
     \\170\\171\\172\\173\\174\\175\\176\\177\\178\\179\
     \\180\\181\\182\\183\\184\\185\\186\\187\\188\\189\
     \\190\\191\\192\\193\\194\\195\\196\\197\\198\\199\
     \\200\\201\\202\\203\\204\\205\\206\\207\\208\\209\
     \\210\\211\\212\\213\\214\\215\\216\\217\\218\\219\
     \\220\\221\\222\\223\\224\\225\\226\\227\\228\\229\
     \\230\\231\\232\\233\\234\\235\\236\\237\\238\\239\
     \\240\\241\\242\\243\\244\\245\\246\\247\\248\\249\
     \\250\\251\\252\\253\\254\\255";

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
    name: String,
    reg_type: String,
    domain: String,
}

#[derive(Debug)]
pub struct Reply
{
    full_name: String,
    host_target: String,
    port: u16,
    txt_data: Vec<String>
}

impl Request
{
    pub fn new(service_flags: ServiceFlags, interface_index: u32, name: String, reg_type: String, domain: String) -> Self
    {
        Request { service_flags, interface_index, name, reg_type, domain }
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.service_flags as u32).to_be_bytes());
        buf.extend_from_slice(&self.interface_index.to_be_bytes());
        buf.extend_from_slice(self.name.as_bytes());
        buf.push(0); // NUL-terminate
        buf.extend_from_slice(self.reg_type.as_bytes());
        buf.push(0); // NUL-terminate
        buf.extend_from_slice(self.domain.as_bytes());
        buf.push(0); // NUL-terminate
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
            return Err("Buffer too short to contain full name".to_string());
        }

        let full_name = Self::cstr_from_buf(&buf[offset..]);
        offset += full_name.len() + 1;

        if offset >= buf.len() {
            return Err("Buffer too short to contain host target".to_string());
        }

        let host_target = Self::cstr_from_buf(&buf[offset..]);
        offset += host_target.len() + 1;

        if offset + 2 >= buf.len()
        {
            return Err("Buffer too short for port".to_string());
        }

        let port = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
        offset += 2;

        if offset + 2 > buf.len()
        {
            return Err("Buffer too short for txtLen".to_string());
        }

        let txt_len = u16::from_be_bytes([buf[offset], buf[offset + 1]]) as usize;
        offset += 2;

        if offset + txt_len > buf.len()
        {
            return Err("Buffer too short for txtRData".to_string());
        }

        let (txt_data, _) = unpack_txt(&buf[offset..offset + txt_len], 0)?;

        Ok(Reply { full_name, host_target, port, txt_data })
    }
}

fn escape_byte(b: u8) -> &'static str
{
    if b < b' '
    {
        let start = (b as usize) * 4;
        return &ESCAPED_BYTE_SMALL[start..start + 4];
    }
    else
    {
        let offset = b - (b'~' + 1);
        let start = (offset as usize) * 4;
        return &ESCAPED_BYTE_LARGE[start..start + 4];
    }
}

pub fn unpack_string(msg: &[u8], off: usize) -> Result<(String, usize), String>
{
    if off + 1 > msg.len()
    {
        return Err("overflow unpacking txt (len byte)".to_string());
    }

    let l = msg[off] as usize;
    let mut off = off + 1;
    if off + l > msg.len()
    {
        return Err("overflow unpacking txt (data)".to_string());
    }

    let mut s = String::new();
    let mut consumed = 0;
    let slice = &msg[off..off + l];

    for (i, &b) in slice.iter().enumerate()
    {
        match b
        {
            b'"' | b'\\' =>
            {
                if consumed == 0
                {
                    s.reserve(l * 2);
                }

                s.push_str(&String::from_utf8_lossy(&slice[consumed..i]));
                s.push('\\');
                s.push(b as char);
                consumed = i + 1;
            }
            b if b < b' ' || b > b'~' =>
            {
                if consumed == 0
                {
                    s.reserve(l * 2);
                }

                s.push_str(&String::from_utf8_lossy(&slice[consumed..i]));
                s.push_str(escape_byte(b));
                consumed = i + 1;
            }
            _ =>
            {
                // no escaping needed
            }
        }
    }

    if consumed == 0
    {
        // no escaping needed
        return Ok((String::from_utf8_lossy(&slice).to_string(), off + l));
    }

    s.push_str(&String::from_utf8_lossy(&slice[consumed..]));

    return Ok((s, off + l));
}

pub fn unpack_txt(msg: &[u8], offset: usize) -> Result<(Vec<String>, usize), String>
{
    let mut offset = offset;
    let mut txts = Vec::new();

    while offset < msg.len()
    {
        match unpack_string(msg, offset)
        {
            Ok((txt, new_offset)) =>
            {
                txts.push(txt);
                offset = new_offset;
            },
            Err(e) =>
            {
                // If we haven't read any strings, return the error
                if txts.is_empty()
                {
                    return Err(e);
                }
                else
                {
                    // Otherwise, stop and return what we have so far
                    break;
                }
            }
        }
    }

    Ok((txts, offset))
}
