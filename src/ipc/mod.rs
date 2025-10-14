use log::{ debug, error };
use std::io;
use tokio::net::{ UnixStream, unix::{OwnedReadHalf, OwnedWriteHalf}, };
use std::net::IpAddr;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task;
use tokio_util::sync::CancellationToken;

use crate::mdnsresponder_error::InternalError;

mod header;
mod operation;

const SOCKET_PATH: &str = "/var/run/mDNSResponder";

pub struct Ipc
{
    listen_task: task::JoinHandle<()>,
    cancel_token: CancellationToken,
    write_socket: OwnedWriteHalf,
}

impl Ipc
{
    pub async fn new(event_sender: mpsc::Sender<super::MDnsResponderEvent>) -> io::Result<Self>
    {
        let stream = match UnixStream::connect(SOCKET_PATH).await
        {
            Ok(s) => s,
            Err(e) =>
            {
                error!("Failed to connect to mDNSResponder socket: {}", e);
                return Err(e);
            }
        };

        let cancel_token = CancellationToken::new();
        let (read_socket, write_socket) = stream.into_split();

        let listen_task = task::spawn(Self::listener(
            read_socket,
            cancel_token.clone(),
            event_sender,
        ));

        return Ok(Ipc
        {
            listen_task,
            cancel_token,
            write_socket,
        });
    }

    pub async fn close(self)
    {
        debug!("Closing IPC connection to mDNSResponder");
        self.cancel_token.cancel();
        self.listen_task
            .await
            .expect("Failed to join IPC listener task");
    }

    async fn listener(
        read: OwnedReadHalf,
        task_cancel_token: CancellationToken,
        event_sender: mpsc::Sender<super::MDnsResponderEvent>,
    )
    {
        debug!("Starting IPC listener for mDNSResponder socket");

        let mut buffer: Vec<u8> = Vec::new();

        loop
        {
            select!
            {
                _ = task_cancel_token.cancelled() =>
                {
                    log::debug!("Cancellation token triggered, stopping IPC listener.");
                    break;
                }
                _ = read.readable() =>
                {
                    let mut read_buffer = [0u8; 2048];
                    match read.try_read(&mut read_buffer)
                    {
                        Ok(0) =>
                        {
                            debug!("No data read, socket may be closed");
                            break;
                        }
                        Ok(n) =>
                        {
                            debug!("Read {} bytes from IPC socket", n);

                            buffer.extend_from_slice(&read_buffer[..n]);

                            // Try to parse as many complete frames as possible.
                            let mut pos = 0;
                            while pos < buffer.len()
                            {
                                match Self::parse_frame(&buffer[pos..], &event_sender).await
                                {
                                    Ok(frame_size) =>
                                    {
                                        debug!("Parsed frame of size {}", frame_size);
                                        pos += frame_size;
                                    }
                                    Err(InternalError::IncompleteFrame) =>
                                    {
                                        debug!("Incomplete frame, waiting for more data");
                                        break;
                                    }
                                    Err(e) =>
                                    {
                                        error!("Error parsing frame: {}", e);
                                        // Clear the entire buffer on parsing error
                                        buffer.clear();
                                        pos = 0;
                                        break;
                                    }
                                }
                            }

                            if pos > 0
                            {
                                debug!("Processed {} bytes, removing from buffer", pos);
                                buffer.drain(0..pos);
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock =>
                        {
                            debug!("WouldBlock error occurred, socket is not ready for reading");
                            continue;
                        }
                        Err(e) =>
                        {
                            error!("Error reading from mDNSResponder socket: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn write(&mut self, buf: &[u8]) -> io::Result<usize>
    {
        self.write_socket
            .writable()
            .await
            .expect("Failed to set writable on stream");

        match self.write_socket.try_write(buf)
        {
            Ok(n) =>
            {
                debug!("Successfully wrote {} bytes to mDNSResponder socket", n);
                return Ok(n);
            }
            Err(e) =>
            {
                error!("Failed to write to mDNSResponder socket: {}", e);
                return Err(e);
            }
        }
    }

    pub async fn write_browse_request(
        &mut self,
        service_type: String,
        service_domain: String,
    ) -> Result<u64, io::Error>
    {
        let request = operation::browse::Request::new(
            operation::browse::ServiceFlags::none,
            0, // Interface index, set to 0 for default
            service_type,
            service_domain,
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::no_err_sd as u32,
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

    pub async fn write_cancel_request(&mut self, context: u64) -> Result<(), io::Error>
    {
        let header = header::IpcMessageHeader::new(
            1, // Version
            0, // No data
            header::IpcFlags::no_err_sd as u32,
            header::Operation::Request(header::request::RequestOperation::Cancel),
            context,
            0, // Registration index, set to 0 for default
        );

        let header_buf = header.to_bytes();

        self.write(&header_buf).await?;

        return Ok(());
    }

    pub async fn write_resolve_request(
        &mut self,
        service_name: String,
        reg_type: String,
        service_domain: String,
    ) -> Result<u64, io::Error>
    {
        let request = operation::resolve::Request::new(
            operation::resolve::ServiceFlags::none,
            0, // Interface index, set to 0 for default
            service_name,
            reg_type,
            service_domain,
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::no_err_sd as u32,
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

    pub async fn write_addrinfo_request(
        &mut self,
        protocol: super::Protocol,
        hostname: String
    ) -> Result<u64, io::Error>
    {
        let request = operation::addrinfo::Request::new(
            operation::addrinfo::ServiceFlags::none,
            0, // Interface index, set to 0 for default
            protocol.into(),
            hostname,
        );

        let request_buf = request.to_bytes();

        let header = header::IpcMessageHeader::new(
            1, // Version
            request_buf.len() as u32,
            header::IpcFlags::no_err_sd as u32,
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

    async fn parse_frame(
        buf: &[u8],
        event_sender: &mpsc::Sender<super::MDnsResponderEvent>,
    ) -> Result<usize, InternalError>
    {
        match header::IpcMessageHeader::from(&buf)
        {
            Ok(header) =>
            {
                debug!("Received IPC message: {:?}", header);

                match header.operation
                {
                    header::Operation::Reply(reply) => match reply
                    {
                        header::reply::ReplyOperation::Browse =>
                        {
                            return Self::parse_browse_reply(buf, header.data_length, event_sender)
                                .await;
                        }
                        header::reply::ReplyOperation::Resolve =>
                        {
                            return Self::parse_resolve_reply(
                                buf,
                                header.data_length,
                                event_sender,
                            )
                            .await;
                        }
                        header::reply::ReplyOperation::AddressInfo =>
                        {
                            return Self::parse_address_info_reply(buf, header.data_length, event_sender)
                                .await;
                        }
                        _ =>
                        {
                            debug!("Received other reply operation: {:?}", reply);
                            return Err(InternalError::FrameParsingFailed);
                        }
                    },
                    _ =>
                    {
                        debug!("Received non-reply IPC message");
                        return Err(InternalError::FrameParsingFailed);
                    }
                }
            }
            Err(e) =>
            {
                error!("Failed to parse IPC message header: {}", e);
                return Err(InternalError::FrameParsingFailed);
            }
        }
    }

    async fn parse_browse_reply(
        buf: &[u8],
        data_length: u32,
        event_sender: &mpsc::Sender<super::MDnsResponderEvent>,
    ) -> Result<usize, InternalError>
    {
        let start_pos = header::IPC_HEADER_SIZE;
        let stop_pos = start_pos + data_length as usize;

        if stop_pos > buf.len()
        {
            debug!("Incomplete frame (fragmentation): need {} bytes, have {}", stop_pos, buf.len());
            return Err(InternalError::IncompleteFrame);
        }

        let browse_reply = match operation::browse::Reply::from_bytes(&buf[start_pos..stop_pos])
        {
            Ok(reply) => reply,
            Err(e) =>
            {
                error!("Failed to parse browse reply: {}", e);
                return Err(InternalError::FrameParsingFailed);
            }
        };

        let is_add = browse_reply.is_add();

        let service = super::Service
        {
            name: browse_reply.service_name,
            service_type: browse_reply.service_type,
            domain: browse_reply.service_domain,
        };

        if is_add
        {
            if let Err(e) = event_sender
                .send(super::MDnsResponderEvent::ServiceAdded(service))
                .await
            {
                error!("Failed to send service added notification: {}", e);
            }
        }
        else
        {
            if let Err(e) = event_sender
                .send(super::MDnsResponderEvent::ServiceRemoved(service))
                .await
            {
                error!("Failed to send service removed notification: {}", e);
            }
        }

        return Ok(header::IPC_HEADER_SIZE + data_length as usize);
    }

    async fn parse_resolve_reply(
        buf: &[u8],
        data_length: u32,
        event_sender: &mpsc::Sender<super::MDnsResponderEvent>,
    ) -> Result<usize, InternalError>
    {
        let start_pos = header::IPC_HEADER_SIZE;
        let stop_pos = start_pos + data_length as usize;

        if stop_pos > buf.len() {
            debug!("Incomplete frame (fragmentation): need {} bytes, have {}", stop_pos, buf.len());
            return Err(InternalError::IncompleteFrame);
        }

        let resolve_reply = match operation::resolve::Reply::from_bytes(&buf[start_pos..stop_pos])
        {
            Ok(reply) => reply,
            Err(e) =>
            {
                error!("Failed to parse resolve reply: {}", e);
                return Err(InternalError::FrameParsingFailed);
            }
        };

        let resolved = super::Resolved
        {
            full_name: resolve_reply.full_name,
            host_target: resolve_reply.host_target,
            port: resolve_reply.port,
            txt_data: resolve_reply.txt_data,
        };

        if let Err(e) = event_sender
            .send(super::MDnsResponderEvent::ServiceResolved(resolved))
            .await
        {
            error!("Failed to send service resolved notification: {}", e);
        }

        return Ok(header::IPC_HEADER_SIZE + data_length as usize);
    }

    async fn parse_address_info_reply(
        buf: &[u8],
        data_length: u32,
        event_sender: &mpsc::Sender<super::MDnsResponderEvent>,
    ) -> Result<usize, InternalError>
    {
        let start_pos = header::IPC_HEADER_SIZE;
        let stop_pos = start_pos + data_length as usize;

        if stop_pos > buf.len()
        {
            debug!("Incomplete frame (fragmentation): need {} bytes, have {}", stop_pos, buf.len());
            return Err(InternalError::IncompleteFrame);
        }

        let addrinfo_reply = match operation::addrinfo::Reply::from_bytes(&buf[start_pos..stop_pos])
        {
            Ok(reply) => reply,
            Err(e) =>
            {
                error!("Failed to parse address info reply: {}", e);
                return Err(InternalError::FrameParsingFailed);
            }
        };

        let ip_addr = match addrinfo_reply.rdata.len()
        {
            4 =>
            {
                IpAddr::from([
                    addrinfo_reply.rdata[0],
                    addrinfo_reply.rdata[1],
                    addrinfo_reply.rdata[2],
                    addrinfo_reply.rdata[3],
                ])
            }
            16 =>
            {
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&addrinfo_reply.rdata[..16]);
                IpAddr::from(octets)
            }
            _ =>
            {
                error!("Unexpected rdata length for IP address: {}", addrinfo_reply.rdata.len());
                return Err(InternalError::FrameParsingFailed);
            }
        };

        let addr_info = super::AddressInfo
        {
            hostname: addrinfo_reply.name,
            address: ip_addr,
        };

        if let Err(e) = event_sender
            .send(super::MDnsResponderEvent::AddressInfoResolved(addr_info))
            .await
        {
            error!("Failed to send address info notification: {}", e);
        }

        return Ok(header::IPC_HEADER_SIZE + data_length as usize);
    }
}
