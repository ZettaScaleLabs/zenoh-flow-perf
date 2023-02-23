//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//
use crate::types::TCPEndpoint;
use crate::{
    IP_HDR_BLOCK_LEN, TCP_HDR_LEN, ZENOH_FRAME_SIZE_LEN, ZENOH_MULTICAST_PORT, ZENOH_UNICAST_PORT,
};
use async_std::sync::Arc;
use sniffglue::centrifuge::*;
use sniffglue::structs::ether::Ether;
use sniffglue::structs::ipv4::IPv4;
use sniffglue::structs::raw::Raw;
use sniffglue::structs::tcp::TCP;
use sniffglue::structs::udp::UDP;
// use sniffglue::structs::ipv6::IPv6;
use std::collections::HashMap;
use zenoh_buffers::ZSlice;

pub fn read(data: &[u8]) -> Option<(ZSlice, usize)> {
    let mut buffer: Vec<u8> = Vec::new();
    let length = u16::from_le_bytes([data[0], data[1]]) as usize;
    if length > data.len() || length == 0 {
        return None;
    }
    buffer.extend_from_slice(&data[ZENOH_FRAME_SIZE_LEN..ZENOH_FRAME_SIZE_LEN + length]);
    let next = ZENOH_FRAME_SIZE_LEN + length;
    Some((ZSlice::make(Arc::new(buffer), 0, length).unwrap(), next))
}

pub fn read_messages(data: Vec<u8>) -> Vec<ZSlice> {
    let mut slices = Vec::new();
    let mut current = 0;
    while current < data.len() {
        if let Some((slice, next)) = read(&data[current..]) {
            log::trace!("[Read Messages] ZSlice {:?}", slice);
            slices.push(slice);
            current += next;
        } else {
            break;
        }
    }
    slices
}

pub fn parse_pcap(
    file_path: &std::path::PathBuf,
) -> (HashMap<(TCPEndpoint, TCPEndpoint), Vec<u8>>, Vec<Vec<u8>>) {
    let mut tcp_sessions: HashMap<(TCPEndpoint, TCPEndpoint), Vec<u8>> = HashMap::new();
    let mut udp_scout: Vec<Vec<u8>> = Vec::new();

    let mut cap = sniffglue::sniff::open_file(file_path.to_str().expect("Error when parsing path"))
        .expect("Unable to read file");
    let link_type = cap.datalink();
    let datalink =
        sniffglue::link::DataLink::from_linktype(link_type).expect("Unable to find datalink");

    // First let's get out TCP and UDP traffic for Zenoh
    while let Ok(Some(packet)) = cap.next_pkt() {
        let pkt = parse(&datalink, &packet.data);
        log::trace!("Packet: {pkt:?}");
        match pkt {
            Raw::Ether(_, ether) => {
                match ether {
                    Ether::IPv4(ip_hdr, ip_pkt) => {
                        let src_addr = ip_hdr.source_addr;
                        let dst_addr = ip_hdr.dest_addr;
                        let ip_hdr_len = (ip_hdr.ihl as usize) * IP_HDR_BLOCK_LEN;
                        let total_len = ip_hdr.length as usize;

                        match ip_pkt {
                            IPv4::TCP(tcp_hdr, tcp_pkt) => {
                                // check the ports first
                                let src_port = tcp_hdr.source_port;
                                let dst_port = tcp_hdr.dest_port;

                                if src_port == ZENOH_UNICAST_PORT || dst_port == ZENOH_UNICAST_PORT
                                {
                                    // this is part of a Zenoh session
                                    let src_endpoint = TCPEndpoint {
                                        addr: src_addr.into(),
                                        port: src_port,
                                    };
                                    let dst_endpoint = TCPEndpoint {
                                        addr: dst_addr.into(),
                                        port: dst_port,
                                    };

                                    let data_len = total_len - ip_hdr_len - TCP_HDR_LEN;

                                    match tcp_sessions
                                        .get_mut(&(src_endpoint.clone(), dst_endpoint.clone()))
                                    {
                                        Some(tcp_session_data) => {
                                            if let TCP::Binary(tcp_data) = tcp_pkt {
                                                let payload = &tcp_data[..data_len];
                                                if payload.len() > 0 {
                                                    tcp_session_data.extend_from_slice(payload);
                                                }
                                            }
                                        }
                                        None => {
                                            if let TCP::Binary(tcp_data) = tcp_pkt {
                                                let payload = &tcp_data[..data_len];
                                                if payload.len() > 0 {
                                                    let mut tcp_session_data = Vec::new();
                                                    tcp_session_data.extend_from_slice(payload);
                                                    tcp_sessions.insert(
                                                        (src_endpoint, dst_endpoint),
                                                        tcp_session_data,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            IPv4::UDP(udp_hdr, udp_pkt) => {
                                let src_port = udp_hdr.source_port;
                                let dst_port = udp_hdr.dest_port;

                                // If it is UDP check that the port used is the zenoh scouting port
                                if dst_port == ZENOH_MULTICAST_PORT {
                                    // then store the payload
                                    if let UDP::Binary(udp_data) = udp_pkt {
                                        udp_scout.push(udp_data);
                                    }
                                }

                                // Otherwise it may be a zenoh UDP session?
                                if src_port == ZENOH_UNICAST_PORT || dst_port == ZENOH_UNICAST_PORT
                                {
                                    // TODO
                                }
                            }
                            _ => log::debug!(
                                "Only supports TCP/UDP packets, this is not TCP/UDP, skipping"
                            ),
                        }
                    }
                    Ether::IPv6(_ip_hdr, _ip_pkt) => {
                        // This is the same as IPv4... to do later
                    }
                    _ => log::debug!("Only supports IP packets, this is not IP, skipping"),
                }
            }
            _ => log::debug!("Only supports Ethernet frames, this is not ethernet, skipping"),
        }
    }

    (tcp_sessions, udp_scout)
}
