use core::fmt;
use std::io;

pub mod reply;
pub mod request;

#[derive(Debug)]
pub enum Operation
{
    Request(request::RequestOperation),
    Reply(reply::ReplyOperation),
}

pub enum IpcFlags
{
    NoReply = 0x0,
    TrailingTlvs = 0x2,
    NoErrSd = 0x4,
}

pub const IPC_HEADER_SIZE: usize = 28;

pub struct IpcMessageHeader
{
    pub version: u32,
    pub data_length: u32,
    pub ipc_flags: u32,
    pub operation: Operation,
    pub client_context: u64,
    pub reg_index: u32,
}

impl IpcMessageHeader
{
    pub fn from(buf: &[u8]) -> io::Result<Self>
    {
        if buf.len() < IPC_HEADER_SIZE
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Buffer too short for IPC message header",
            ));
        }

        let version = u32::from_be_bytes(buf[0..4].try_into().unwrap());
        let data_length = u32::from_be_bytes(buf[4..8].try_into().unwrap());
        let ipc_flags = u32::from_be_bytes(buf[8..12].try_into().unwrap());
        let operation_num = u32::from_be_bytes(buf[12..16].try_into().unwrap());
        let client_context = u64::from_be_bytes(buf[16..24].try_into().unwrap());
        let reg_index = u32::from_be_bytes(buf[24..28].try_into().unwrap());

        let operation;
        if operation_num >= reply::REPLY_OPERATION_START
        {
            let reply_operation =
                reply::ReplyOperation::from_u32(operation_num).ok_or_else(||
                    {
                        io::Error::new(io::ErrorKind::InvalidData, "Invalid reply operation")
                    }
                )?;
            operation = Operation::Reply(reply_operation);
        }
        else
        {
            let request_operation =
                request::RequestOperation::from_u32(operation_num).ok_or_else(||
                    {
                        io::Error::new(io::ErrorKind::InvalidData, "Invalid request operation")
                    }
                )?;
            operation = Operation::Request(request_operation);
        }

        return Ok(IpcMessageHeader
        {
            version,
            data_length,
            ipc_flags,
            operation,
            client_context,
            reg_index,
        });
    }

    pub fn new(
        version: u32,
        data_length: u32,
        ipc_flags: u32,
        operation: Operation,
        client_context: u64,
        reg_index: u32,
    ) -> Self
    {
        return IpcMessageHeader
        {
            version,
            data_length,
            ipc_flags,
            operation,
            client_context,
            reg_index,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8>
    {
        let mut buf = Vec::with_capacity(20);

        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.data_length.to_be_bytes());
        buf.extend_from_slice(&self.ipc_flags.to_be_bytes());
        buf.extend_from_slice(&match &self.operation
            {
                Operation::Request(op) => op.to_u32().to_be_bytes(),
                Operation::Reply(op) => op.to_u32().to_be_bytes(),
            }
        );
        buf.extend_from_slice(&self.client_context.to_be_bytes());
        buf.extend_from_slice(&self.reg_index.to_be_bytes());

        return buf;
    }
}

impl std::fmt::Debug for IpcMessageHeader
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("IpcMessageHeader")
            .field("version", &self.version)
            .field("data_length", &self.data_length)
            .field("ipc_flags", &self.ipc_flags)
            .field("operation", &self.operation)
            .field("client_context", &self.client_context)
            .field("reg_index", &self.reg_index)
            .finish()
    }
}
