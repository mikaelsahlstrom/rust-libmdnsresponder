use tokio::net::{ UnixStream, unix::{ OwnedReadHalf, OwnedWriteHalf } };
use tokio::task;
use tokio_util::sync::CancellationToken;
use std::io;
use log::{ debug, error };
use tokio::select;

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
    pub async fn new() -> io::Result<Self>
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

        let listen_task = task::spawn(Self::listener(read_socket, cancel_token.clone()));

        return Ok(Ipc { listen_task, cancel_token, write_socket });
    }

    pub async fn close(self)
    {
        debug!("Closing IPC connection to mDNSResponder");
        self.cancel_token.cancel();
        debug!("Cancellation token has been triggered, waiting for listener task to finish");
        self.listen_task.await.expect("Failed to join IPC listener task");
        debug!("IPC listener task has been cancelled and joined");
    }

    async fn listener(mut read: OwnedReadHalf, task_cancel_token: CancellationToken)
    {
        debug!("Starting IPC listener for mDNSResponder socket");

        loop
        {
            debug!("Before select in read loop");
            select!
            {
                _ = task_cancel_token.cancelled() =>
                {
                    log::debug!("Cancellation token triggered, stopping IPC listener.");
                    break;
                }
                _ = read.readable() =>
                {
                    debug!("Socket is readable");
                    let mut buf = [0u8; 1024];
                    match read.try_read(&mut buf)
                    {
                        Ok(0) =>
                        {
                            debug!("No data read, socket may be closed");
                            break;
                        }
                        Ok(n) =>
                        {
                            debug!("Read {} bytes from IPC socket", n);

                            let mut pos = 0;
                            while pos < n
                            {
                                let frame_size = Self::parse_frame(&buf[pos..n]);
                                if frame_size == 0
                                {
                                    debug!("No more complete frames to parse");
                                    break;
                                }

                                pos += frame_size;
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

    async fn write(&mut self, buf: &[u8])
    {
        self.write_socket.writable().await.expect("Failed to set writable on stream");
        match self.write_socket.try_write(buf)
        {
            Ok(_) => debug!("Successfully wrote to mDNSResponder socket"),
            Err(e) => error!("Failed to write to mDNSResponder socket: {}", e),
        }
    }

    pub async fn write_browse_request(&mut self, service_type: String, service_domain: String)
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

        debug!("Writing browse request to mDNSResponder: {:?}", buf);

        self.write(&buf).await;
    }

    fn parse_frame(buf: &[u8]) -> usize
    {
        match header::IpcMessageHeader::from(&buf[..buf.len()])
        {
            Ok(header) =>
            {
                debug!("Received IPC message: {:?}", header);

                match header.operation
                {
                    header::Operation::Reply(reply) =>
                    {
                        match reply
                        {
                            header::reply::ReplyOperation::Browse =>
                            {
                                debug!("Received Browse Reply");
                                let start_pos = header::IPC_HEADER_SIZE;
                                let stop_pos = start_pos + header.data_length as usize;
                                let browse_reply = operation::browse::Reply::from_bytes(&buf[start_pos..stop_pos]);
                                debug!("Parsed Browse Reply: {:?}", browse_reply);
                                return header::IPC_HEADER_SIZE + header.data_length as usize;
                            }
                            _ =>
                            {
                                debug!("Received other reply operation: {:?}", reply);
                                return 0;
                            }
                        }
                    }
                    _ =>
                    {
                        debug!("Received non-reply IPC message");
                        return 0;
                    }
                }
            }
            Err(e) =>
            {
                error!("Failed to parse IPC message header: {}", e);
                return 0;
            }
        }
    }
}
