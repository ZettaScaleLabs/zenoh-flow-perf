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

use analyze_zenoh_pcap::analysis::analyze_zenoh_tcp;
use analyze_zenoh_pcap::utils::parse_pcap;
use clap::Parser;
use prettytable::row;
use prettytable::Table;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Analyzer {
    #[clap(name = "Capture file path", help = "Capture to be analyzed")]
    capture_file: std::path::PathBuf,
}

#[async_std::main]
async fn main() {
    env_logger::try_init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    )
    .unwrap_or_else(|_| log::warn!("`env_logger` already initialized"));

    let args = Analyzer::parse();

    let (tcp_sessions, udp_scout) = parse_pcap(&args.capture_file);

    // UDP Scouting Analysis
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

    // Analyze Zenoh's TCP sessions
    let zenoh_sessions = analyze_zenoh_tcp(tcp_sessions);

    let mut transport_table = Table::new();
    transport_table.add_row(row![
        "From IP",
        "From Port",
        "To IP",
        "To Port",
        "Total bytes",
        "Overhead",
        "# Frames (bytes)",
        "# InitSyn (bytes)",
        "# InitAck (bytes)",
        "# OpenSyn (bytes)",
        "# OpenAck (bytes)",
        "# Join (bytes)",
        "# Close (bytes)",
        "# KeepAlive (bytes)",
        "# Tot TxMessages"
    ]);

    let mut proto_table = Table::new();
    proto_table.add_row(row![
        "From IP",
        "From Port",
        "To IP",
        "To Port",
        "Total Payload (data)",
        "Total Payload (query)",
        "# Data (bytes)",
        "# Unit (bytes)",
        "# Pull (bytes)",
        "# Query (bytes)",
        "# Declare (bytes)",
        "# LinkState (bytes)"
    ]);

    for ((src, dst), stats) in zenoh_sessions {
        proto_table.add_row(row![
            format!("{}", src.addr),
            format!("{}", src.port),
            format!("{}", dst.addr),
            format!("{}", dst.port),
            format!("{}", stats.zenoh.total_data_payload),
            format!("{}", stats.zenoh.total_query_paylaod),
            format!(
                "{} ({})",
                stats.zenoh.total_data, stats.zenoh.total_data_bytes
            ),
            format!(
                "{} ({})",
                stats.zenoh.total_unit, stats.zenoh.total_unit_bytes
            ),
            format!(
                "{} ({})",
                stats.zenoh.total_pull, stats.zenoh.total_pull_bytes
            ),
            format!(
                "{} ({})",
                stats.zenoh.total_query, stats.zenoh.total_query_bytes
            ),
            format!(
                "{} ({})",
                stats.zenoh.total_declare, stats.zenoh.total_declare_bytes
            ),
            format!(
                "{} ({})",
                stats.zenoh.total_linkstate, stats.zenoh.total_linkstate_bytes
            )
        ]);

        transport_table.add_row(row![
            format!("{}", src.addr),
            format!("{}", src.port),
            format!("{}", dst.addr),
            format!("{}", dst.port),
            format!("{}", stats.transport.total_bytes),
            format!("{}", stats.overhead),
            format!(
                "{} ({})",
                stats.transport.total_frames, stats.transport.total_frames_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_initsyn, stats.transport.total_initsyn_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_initack, stats.transport.total_initack_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_opensyn, stats.transport.total_opensyn_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_openack, stats.transport.total_openack_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_join, stats.transport.total_join_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_close, stats.transport.total_close_bytes
            ),
            format!(
                "{} ({})",
                stats.transport.total_keepalive, stats.transport.total_keepalive_bytes
            ),
            format!("{}", stats.transport.total_tx_msgs)
        ]);
    }

    println!("Protocol:");
    proto_table.printstd();

    println!("Transport:");
    transport_table.printstd();
}
