use log::{ debug, error };
use std::io;
use tokio::net::{ UnixStream, unix::OwnedReadHalf, };
use tokio::select;
use tokio::sync::mpsc;
use tokio::task;
use tokio_util::sync::CancellationToken;

use crate::mdnsresponder_error::InternalError;

mod header;
mod operation;
mod write;
mod parse;

const SOCKET_PATH: &str = "/var/run/mDNSResponder";

pub struct Ipc
{
    listen_task: task::JoinHandle<()>,
    cancel_token: CancellationToken,
    pub write: write::IpcWriter,
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
            write: write::IpcWriter::new(write_socket),
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
                            buffer.extend_from_slice(&read_buffer[..n]);

                            // Try to parse as many complete frames as possible.
                            Self::try_parse_frame(&event_sender, &mut buffer).await;
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

    async fn try_parse_frame(
        event_sender: &mpsc::Sender<super::MDnsResponderEvent>,
        buffer: &mut Vec<u8>)
    {
        let mut pos = 0;
        while pos < buffer.len()
        {
            match Self::parse_frame(&buffer[pos..], &event_sender).await
            {
                Ok(frame_size) =>
                {
                    pos += frame_size;
                }
                Err(InternalError::IncompleteFrame) =>
                {
                    debug!("Incomplete frame, waiting for more data");
                    break;
                }
                Err(InternalError::MDnsResponderError((code, size))) =>
                {
                    error!("mDNSResponder returned error code: {:?}", code);
                    // Skip this frame and continue parsing
                    pos += size;
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

    async fn parse_frame(
        buf: &[u8],
        event_sender: &mpsc::Sender<super::MDnsResponderEvent>,
    ) -> Result<usize, InternalError>
    {
        match header::IpcMessageHeader::from(&buf)
        {
            Ok(header) =>
            {
                match header.operation
                {
                    header::Operation::Reply(reply) => match reply
                    {
                        header::reply::ReplyOperation::Browse =>
                        {
                            return parse::browse_reply(buf, header.data_length, event_sender)
                                .await;
                        }
                        header::reply::ReplyOperation::Resolve =>
                        {
                            return parse::resolve_reply(
                                buf,
                                header.data_length,
                                event_sender,
                            )
                            .await;
                        }
                        header::reply::ReplyOperation::AddressInfo =>
                        {
                            return parse::address_info_reply(buf, header.data_length, event_sender)
                                .await;
                        }
                        header::reply::ReplyOperation::RegisterService =>
                        {
                            return parse::register_service_reply(buf, header.data_length)
                                .await;
                        }
                        _ =>
                        {
                            error!("Received other reply operation: {:?}", reply);
                            return Err(InternalError::FrameParsingFailed);
                        }
                    },
                    _ =>
                    {
                        error!("Received non-reply IPC message");
                        return Err(InternalError::FrameParsingFailed);
                    }
                }
            }
            Err(InternalError::IncompleteFrame) =>
            {
                error!("Incomplete frame (fragmentation)");
                return Err(InternalError::IncompleteFrame);
            }
            Err(e) =>
            {
                error!("Failed to parse IPC message header: {}", e);
                return Err(InternalError::FrameParsingFailed);
            }
        }
    }
}
