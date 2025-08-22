pub mod browse;

pub enum IpcOperation
{
    BrowseRequest(browse::Request),
}

impl IpcOperation
{
    pub fn to_bytes(&self) -> Vec<u8>
    {
        match self {
            IpcOperation::BrowseRequest(req) => req.to_bytes(),
        }
    }
}
