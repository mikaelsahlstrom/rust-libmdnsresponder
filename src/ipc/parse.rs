use tokio::sync::mpsc;
use log::{ debug, error };
use std::net::IpAddr;

use crate::MDnsResponderEvent;
use crate::mdnsresponder_error::{ InternalError, ErrorCode };
use crate::ipc::{ header, operation };
use crate::Service;
use crate::Resolved;
use crate::AddressInfo;

pub async fn browse_reply(
    buf: &[u8],
    data_length: u32,
    event_sender: &mpsc::Sender<MDnsResponderEvent>,
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

    if browse_reply.header.error != 0
    {
        error!("Browse reply contains error code: {}", browse_reply.header.error);
        return Err(InternalError::MDnsResponderError((ErrorCode::from_i32(browse_reply.header.error as i32),
                                                        header::IPC_HEADER_SIZE + data_length as usize)));
    }

    let is_add = browse_reply.is_add();

    let service = Service
    {
        interface_index: browse_reply.header.interface_index,
        name: browse_reply.service_name,
        service_type: browse_reply.service_type,
        domain: browse_reply.service_domain,
    };

    if is_add
    {
        if let Err(e) = event_sender
            .send(MDnsResponderEvent::ServiceAdded(service))
            .await
        {
            error!("Failed to send service added notification: {}", e);
        }
    }
    else
    {
        if let Err(e) = event_sender
            .send(MDnsResponderEvent::ServiceRemoved(service))
            .await
        {
            error!("Failed to send service removed notification: {}", e);
        }
    }

    return Ok(header::IPC_HEADER_SIZE + data_length as usize);
}

pub async fn resolve_reply(
    buf: &[u8],
    data_length: u32,
    event_sender: &mpsc::Sender<MDnsResponderEvent>,
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

    if resolve_reply.header.error != 0
    {
        error!("Resolve reply contains error code: {}", resolve_reply.header.error);
        return Err(InternalError::MDnsResponderError((ErrorCode::from_i32(resolve_reply.header.error as i32),
                                                        header::IPC_HEADER_SIZE + data_length as usize)));
    }

    let resolved = Resolved
    {
        interface_index: resolve_reply.header.interface_index,
        full_name: resolve_reply.full_name,
        host_target: resolve_reply.host_target,
        port: resolve_reply.port,
        txt_data: resolve_reply.txt_data,
    };

    if let Err(e) = event_sender
        .send(MDnsResponderEvent::ServiceResolved(resolved))
        .await
    {
        error!("Failed to send service resolved notification: {}", e);
    }

    return Ok(header::IPC_HEADER_SIZE + data_length as usize);
}


pub async fn address_info_reply(
    buf: &[u8],
    data_length: u32,
    event_sender: &mpsc::Sender<MDnsResponderEvent>,
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

    if addrinfo_reply.header.error != 0
    {
        error!("Address info reply contains error code: {}", addrinfo_reply.header.error);
        return Err(InternalError::MDnsResponderError((ErrorCode::from_i32(addrinfo_reply.header.error as i32),
                                                        header::IPC_HEADER_SIZE + data_length as usize)));
    }

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

    let addr_info = AddressInfo
    {
        interface_index: addrinfo_reply.header.interface_index,
        hostname: addrinfo_reply.name,
        address: ip_addr,
    };

    if let Err(e) = event_sender
        .send(MDnsResponderEvent::AddressInfoResolved(addr_info))
        .await
    {
        error!("Failed to send address info notification: {}", e);
    }

    return Ok(header::IPC_HEADER_SIZE + data_length as usize);
}

pub async fn register_service_reply(
    buf: &[u8],
    data_length: u32,
) -> Result<usize, InternalError>
{
    let start_pos = header::IPC_HEADER_SIZE;
    let stop_pos = start_pos + data_length as usize;

    if stop_pos > buf.len()
    {
        debug!("Incomplete frame (fragmentation): need {} bytes, have {}", stop_pos, buf.len());
        return Err(InternalError::IncompleteFrame);
    }

    let register_reply = match operation::register::Reply::from_bytes(&buf[start_pos..stop_pos])
    {
        Ok(reply) => reply,
        Err(e) =>
        {
            error!("Failed to parse register service reply: {}", e);
            return Err(InternalError::FrameParsingFailed);
        }
    };

    if register_reply.header.error != 0
    {
        error!("Register service reply contains error code: {}", register_reply.header.error);
        return Err(InternalError::MDnsResponderError((ErrorCode::from_i32(register_reply.header.error as i32),
                                                        header::IPC_HEADER_SIZE + data_length as usize)));
    }

    return Ok(header::IPC_HEADER_SIZE + data_length as usize);
}
