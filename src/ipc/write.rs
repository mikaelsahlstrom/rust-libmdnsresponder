use std::io;
use log::error;
use tokio::net::unix::OwnedWriteHalf;

use crate::ipc::header;
use crate::ipc::operation;
use crate::Protocol;

pub struct IpcWriter
{
    write_socket: OwnedWriteHalf,
}

impl IpcWriter
{
    pub fn new(write_socket: OwnedWriteHalf) -> Self
    {
        return IpcWriter
        {
            write_socket,
        };
    }

    pub async fn write(&mut self, buf: &[u8]) -> io::Result<usize>
    {
        self.write_socket
            .writable()
        .await
        .expect("Failed to set writable on stream");

        match self.write_socket.try_write(buf)
        {
            Ok(n) =>
            {
                return Ok(n);
            }
            Err(e) =>
            {
                error!("Failed to write to mDNSResponder socket: {}", e);
                return Err(e);
            }
        }
    }

    pub async fn browse_request(
        &mut self,
        interface_index: u32,
        service_type: String,
        service_domain: String,
    ) -> Result<u64, io::Error>
    {
        let request = operation::browse::Request::new(
            operation::ServiceFlags::None,
            interface_index,
            service_type,
            service_domain,
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::NoErrSd as u32,
            header::Operation::Request(header::request::RequestOperation::Browse),
            rand::random::<u64>(),
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        let mut buf = Vec::with_capacity(header_buf.len() + request_buf.len());
        buf.extend_from_slice(&header_buf);
        buf.extend_from_slice(&request_buf);

        self.write(&buf).await?;

        return Ok(header.client_context);
    }

    pub async fn cancel_request(&mut self, context: u64) -> Result<(), io::Error>
    {
        let header = header::IpcMessageHeader::new(
            1, // Version
            0, // No data
            header::IpcFlags::NoErrSd as u32,
            header::Operation::Request(header::request::RequestOperation::Cancel),
            context,
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        self.write(&header_buf).await?;

        return Ok(());
    }

    pub async fn resolve_request(
        &mut self,
        interface_index: u32,
        service_name: String,
        reg_type: String,
        service_domain: String,
    ) -> Result<u64, io::Error>
    {
        let request = operation::resolve::Request::new(
            operation::ServiceFlags::None,
            interface_index,
            service_name,
            reg_type,
            service_domain,
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::NoErrSd as u32,
            header::Operation::Request(header::request::RequestOperation::Resolve),
            rand::random::<u64>(),
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        let mut buf = Vec::with_capacity(header_buf.len() + request_buf.len());
        buf.extend_from_slice(&header_buf);
        buf.extend_from_slice(&request_buf);

        self.write(&buf).await?;

        return Ok(header.client_context);
    }

    pub async fn addrinfo_request(
        &mut self,
        interface_index: u32,
        protocol: Protocol,
        hostname: String
    ) -> Result<u64, io::Error>
    {
        let request = operation::addrinfo::Request::new(
            operation::ServiceFlags::None,
            interface_index,
            protocol.into(),
            hostname,
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::NoErrSd as u32,
            header::Operation::Request(header::request::RequestOperation::AddressInfo),
            rand::random::<u64>(),
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        let mut buf = Vec::with_capacity(header_buf.len() + request_buf.len());
        buf.extend_from_slice(&header_buf);
        buf.extend_from_slice(&request_buf);

        self.write(&buf).await?;

        return Ok(header.client_context);
    }

    pub async fn register_request(
        &mut self,
        interface_index: u32,
        name: String,
        service_type: String,
        domain: String,
        host: String,
        port: u16,
        txt_data: Vec<String>
    ) -> Result<u64, io::Error>
    {
        let request = operation::register::Request::new(
            operation::ServiceFlags::None,
            interface_index,
            name,
            service_type,
            domain,
            host,
            port,
            txt_data
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::NoErrSd as u32,
            header::Operation::Request(header::request::RequestOperation::RegisterService),
            rand::random::<u64>(),
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        let mut buf = Vec::with_capacity(header_buf.len() + request_buf.len());
        buf.extend_from_slice(&header_buf);
        buf.extend_from_slice(&request_buf);

        self.write(&buf).await?;

        return Ok(header.client_context);
    }

    pub async fn add_record_request(
        &mut self,
        context: u64,
        rrtype: u16,
        rrclass: u16,
        rdata: Vec<u8>,
        ttl: u32
    ) -> Result<(), io::Error>
    {
        let request = operation::addrecord::Request::new(
            operation::ServiceFlags::None,
            rrtype,
            rrclass,
            rdata,
            ttl
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::NoErrSd as u32,
            header::Operation::Request(header::request::RequestOperation::AddRecord),
            context,
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        let mut buf = Vec::with_capacity(header_buf.len() + request_buf.len());
        buf.extend_from_slice(&header_buf);
        buf.extend_from_slice(&request_buf);

        self.write(&buf).await?;

        return Ok(());
    }
}
