use clap::Parser;
use prettytable::row;
use prettytable::Table;
use sniffglue::centrifuge::*;
use sniffglue::structs::ether::Ether;
use sniffglue::structs::ipv4::IPv4;
// use sniffglue::structs::ipv6::IPv6;
use sniffglue::structs::raw::Raw;
use sniffglue::structs::tcp::TCP;
use sniffglue::structs::udp::UDP;
use std::collections::HashMap;
use std::net::IpAddr;
use zenoh_buffers::reader::HasReader;
use zenoh_buffers::reader::Reader;
use zenoh_codec::{RCodec, Zenoh060};
use zenoh_protocol::transport::Frame;
use zenoh_protocol::transport::FramePayload;
use zenoh_protocol::transport::TransportBody;
use zenoh_protocol::transport::TransportMessage;
use zenoh_protocol::zenoh::ZenohBody;
use zenoh_protocol::zenoh::ZenohMessage;
// use zenoh_flow::{
//     model::{
//         descriptor::{DataFlowDescriptor, FlattenDataFlowDescriptor},
//         record::DataFlowRecord,
//     }
// };
use async_std::sync::Arc;
use std::num::NonZeroUsize;
use zenoh_buffers::reader::BacktrackableReader;
use zenoh_buffers::reader::DidntRead;
use zenoh_buffers::{SplitBuffer, ZSlice};

const ZENOH_UNICAST_PORT: u16 = 7447;
const ZENOH_MULTICAST_PORT: u16 = 7446;
const ZENOH_FRAME_SIZE_LEN: usize = 2;
const IP_HDR_BLOCK_LEN: usize = 4;
const TCP_HDR_LEN: usize = 20;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Analyzer {
    #[clap(name = "Capture file path", help = "Capture to be analyzed")]
    capture_file: std::path::PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct TCPEndpoint {
    addr: IpAddr,
    port: u16,
}

struct SizedTransportMessage(TransportMessage, NonZeroUsize);

impl<R> RCodec<SizedTransportMessage, &mut R> for Zenoh060
where
    R: Reader + BacktrackableReader,
{
    type Error = DidntRead;

    fn read(self, reader: &mut R) -> Result<SizedTransportMessage, Self::Error> {
        let len = reader.remaining();
        let message: TransportMessage = self.read(&mut *reader)?;
        let size = NonZeroUsize::new(len - reader.remaining()).unwrap();

        Ok(SizedTransportMessage(message, size))
    }
}

fn read(data: &[u8]) -> Option<(ZSlice, usize)> {
    let mut buffer: Vec<u8> = Vec::new();
    let length = u16::from_le_bytes([data[0], data[1]]) as usize;
    if length > data.len() || length == 0 {
        return None;
    }
    buffer.extend_from_slice(&data[ZENOH_FRAME_SIZE_LEN..ZENOH_FRAME_SIZE_LEN + length]);
    let next = ZENOH_FRAME_SIZE_LEN + length;
    Some((ZSlice::make(Arc::new(buffer), 0, length).unwrap(), next))
}

fn read_messages(data: Vec<u8>) -> Vec<ZSlice> {
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

#[async_std::main]
async fn main() {
    let mut tcp_sessions: HashMap<(TCPEndpoint, TCPEndpoint), Vec<u8>> = HashMap::new();
    let mut udp_scout: Vec<Vec<u8>> = Vec::new();

    env_logger::try_init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    )
    .unwrap_or_else(|_| log::warn!("`env_logger` already initialized"));

    let args = Analyzer::parse();

    let mut cap =
        sniffglue::sniff::open_file(&args.capture_file.to_str().expect("Error when parsing path"))
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

    let mut scout_bytes = 0;
    for u in &udp_scout {
        scout_bytes += u.len();
    }

    let mut scout_table = Table::new();
    println!("Scouting:");
    scout_table.add_row(row!["Souting Messages", "Scouting Bytes"]);
    scout_table.add_row(row![
        format!("{}", udp_scout.len()),
        format!("{scout_bytes}")
    ]);
    scout_table.printstd();

    log::debug!(
        "[Scouting] Found a total of {} UDP scouting datagram",
        udp_scout.len()
    );
    for ((src, dst), data) in &tcp_sessions {
        log::debug!(
            "[Session] From: {src:?} To: {dst:?} - Bytes: {}",
            data.len()
        );
    }

    println!("Protocol:");
    let mut proto_table = Table::new();
    proto_table.add_row(row![
        "From IP",
        "From Port",
        "To IP",
        "To Port",
        "Total bytes",
        "Total Payload (data)",
        "Total Payload (query)",
        "Overhead",
        "# Data",
        "# Unit",
        "# Pull",
        "# Query",
        "# Declare",
        "# LinkState",
    ]);
    // Analyze Zenoh's TCP sessions
    if tcp_sessions.len() > 0 {
        for ((src, dst), data) in tcp_sessions {
            log::debug!("[Session] - Analyzing - From: {src:?} To: {dst:?}");

            let mut total_payload = 0;
            let mut total_query_payload = 0;
            let mut total_data = 0;
            let mut total_unit = 0;
            let mut total_pull = 0;
            let mut total_query = 0;
            let mut total_declare = 0;
            let mut total_linkstate = 0;
            let total_bytes = data.len();

            let mut deserialized: Vec<SizedTransportMessage> = vec![];
            let mut tmsgs: Vec<TransportMessage> = vec![];
            let mut zmsgs: Vec<ZenohMessage> = vec![];

            let codec = Zenoh060::default();

            let slices = read_messages(data);

            for mut slice in slices {
                let mut reader = slice.reader();
                while reader.can_read() {
                    match codec.read(&mut reader) {
                        Ok(msg) => deserialized.push(msg),
                        Err(_e) => (),
                    }
                }
            }

            for smsg in deserialized.drain(..) {
                let SizedTransportMessage(msg, _size) = smsg;
                log::debug!("[Parsing] Zenoh Frame message is {msg:?}");
                match msg.body {
                    TransportBody::Frame(Frame { payload, .. }) => match payload {
                        FramePayload::Messages { mut messages } => zmsgs.append(&mut messages),
                        _ => (),
                    },
                    _ => tmsgs.push(msg),
                }
            }

            for msg in zmsgs {
                match msg.body {
                    ZenohBody::Data(data_msg) => {
                        total_data += 1;
                        total_payload += data_msg.payload.len();
                    }
                    ZenohBody::Unit(_unit_msg) => {
                        total_unit += 1;
                    }
                    ZenohBody::Pull(_pull_msg) => total_pull += 1,
                    ZenohBody::Query(query_msg) => {
                        total_query += 1;
                        match query_msg.body {
                            Some(body) => total_query_payload += body.payload.len(),
                            _ => (),
                        }
                    }
                    ZenohBody::Declare(_declare_msg) => total_declare += 1,
                    ZenohBody::LinkStateList(_linkstate_msg) => total_linkstate += 1,
                }
            }

            let overhead = total_bytes - total_payload - total_query_payload;

            proto_table.add_row(row![
                format!("{}", src.addr),
                format!("{}", src.port),
                format!("{}", dst.addr),
                format!("{}", dst.port),
                format!("{total_bytes}"),
                format!("{total_payload}"),
                format!("{total_query_payload}"),
                format!("{overhead}"),
                format!("{total_data}"),
                format!("{total_unit}"),
                format!("{total_pull}"),
                format!("{total_query}"),
                format!("{total_declare}"),
                format!("{total_linkstate}")
            ]);

            log::debug!("[Session] - From: {src:?} To: {dst:?} Total bytes: {total_bytes} Total Payload: {total_payload} Overhead: {overhead}")
        }
    }

    proto_table.printstd();
}
