#!/usr/bin/env python3

from scapy.all import *
from scapy.contrib.rtps import rtps
import json
import os
import argparse
from prettytable import PrettyTable

def get_rtps_pkt(packet):
    return rtps.RTPS(raw(packet[UDP].payload))


def get_rtps_msg_pkt(packet):
    return rtps.RTPSMessage(raw(packet.payload))


def prepare(pcap_file):
    rtps_pkts = []
    reader = PcapReader(pcap_file)

    for packet in reader:
        # If the packet is not UDP we skip it!

        if UDP not in packet.layers():
            continue

        if not packet[UDP]:
            continue

        # we try to build the RTPS packet and we check the magic value
        # if it does not match we skip
        rtps = get_rtps_pkt(packet)
        if rtps.magic != b'RTPS':
            continue

        rtps = get_rtps_msg_pkt(rtps)

        # if everything is ok we append it
        rtps_pkts.append(rtps)


    return rtps_pkts


def filter_data(rtps_pkts):
    data_pkts = []
    # Filter the DATA messages
    for pkt in rtps_pkts:
        for sm in pkt.submessages:
            if isinstance(sm, rtps.RTPSSubMessage_DATA):
                data_pkts.append(sm)
    return data_pkts


def filter_info_ts(rtps_pkts):
    ts_pkts = []
    # Filter the DATA messages
    for pkt in rtps_pkts:
        for sm in pkt.submessages:
            if isinstance(sm, rtps.RTPSSubMessage_INFO_TS):
                ts_pkts.append(sm)
    return ts_pkts


def filter_discovery(data_pkts):

    w_bytes = 0
    w_count = 0

    p_bytes = 0
    p_count = 0

    m_bytes = 0
    m_count = 0

    r_bytes = 0
    r_count = 0

    # first let's filter the DATA messages

    # then of this DATAs let's only get the discovery infor from DDS:
    #
    # +-------------------+--------------------+-------------------------+
    # | writerEntityIdKey | writerEntityIdKind |       DDS Concept       |
    # +-------------------+--------------------+-------------------------+
    # | 0x100 (256)       | 0xC2 (194)         | Participant             |
    # | 0x3 (3)           | 0xC2 (194)         | Pub Writer (Publisher)  |
    # | 0x3 (4)           | 0xC2 (194)         | Sub Writer (Subscriber) |
    # | 0x200 (512)       | 0xC2               | Message Writer (QoS)    |
    # +-------------------+--------------------+-------------------------+

    for pkt in data_pkts:
        # if the writerEntityKind is not 0xC2 then we skip
        if pkt.writerEntityIdKind != 194:
            continue

        if pkt.writerEntityIdKey == 256:
            # this is a participant discovery
            p_count += 1
            p_bytes += len(raw(pkt))
        elif pkt.writerEntityIdKey == 3:
            # this is a pub discovery
            w_count += 1
            w_bytes += len(raw(pkt))
        elif pkt.writerEntityIdKey == 4:
            # this is a sub discovery
            r_count += 1
            r_bytes += len(raw(pkt))

        elif pkt.writerEntityIdKey == 512:
            # this is a message writer discovery (QoS)
            m_count += 1
            m_bytes += len(raw(pkt))

    tab = PrettyTable()
    tab.field_names = [
            '# Participant Discovery',
            'Bytes Participant Discovery',
            '# Writer Discovery',
            'Bytes Writer Discovery',
            '# Reader Discovery',
            'Bytes Reader Discovery',
            '# QoS Discovery',
            'Bytes QoS Discovery'
        ]

    tab.add_row([
            p_count,
            p_bytes,
            w_count,
            w_bytes,
            r_count,
            r_bytes,
            m_count,
            m_bytes,

        ])

    return tab



def process(rtps_pkts):

    w_bytes = 0
    w_count = 0

    p_bytes = 0
    p_count = 0

    m_bytes = 0
    m_count = 0

    r_bytes = 0
    r_count = 0

    discovery_info_ts_bytes = 0
    discovery_info_ts_count = 0

    data_bytes = 0
    data_count = 0
    data_info_ts_count = 0
    data_info_ts_bytes = 0
    total_payload = 0

    msg_discovery_count = 0
    msg_data_count = 0
    msg_total_count = 0

    msg_bytes_discovery = 0
    msg_bytes_data = 0
    msg_bytes_total = 0
    # Let's parse the messages

    # then of this DATAs let's only get the discovery infor from DDS:
    #
    # +-------------------+--------------------+-------------------------+
    # | writerEntityIdKey | writerEntityIdKind |       DDS Concept       |
    # +-------------------+--------------------+-------------------------+
    # | 0x100 (256)       | 0xC2 (194)         | Participant             |
    # | 0x3 (3)           | 0xC2 (194)         | Pub Writer (Publisher)  |
    # | 0x3 (4)           | 0xC2 (194)         | Sub Writer (Subscriber) |
    # | 0x200 (512)       | 0xC2               | Message Writer (QoS)    |
    # +-------------------+--------------------+-------------------------+

    for pkt in rtps_pkts:
        user_data = False
        # for every submessage

        # quick iteration to check if there are user data as submessages
        # in case there are we set the flag for this message
        for sub_msg in pkt.submessages:
            if isinstance(sub_msg, rtps.RTPSSubMessage_DATA) and sub_msg.writerEntityIdKind != 194:
                user_data = True

        for sub_msg in pkt.submessages:
            # print(f'[{total_count}] {sub_msg} of type {type(sub_msg)}')
            if isinstance(sub_msg, rtps.RTPSSubMessage_DATA):
                # check if it is built-in or user data
                if sub_msg.writerEntityIdKind == 194:
                    # built-in data
                    if sub_msg.writerEntityIdKey == 256:
                        # this is a participant discovery
                        p_count += 1
                        p_bytes += len(raw(sub_msg))
                    elif sub_msg.writerEntityIdKey == 3:
                        # this is a pub discovery
                        w_count += 1
                        w_bytes += len(raw(sub_msg))
                    elif sub_msg.writerEntityIdKey == 4:
                        # this is a sub discovery
                        r_count += 1
                        r_bytes += len(raw(sub_msg))

                    elif sub_msg.writerEntityIdKey == 512:
                        # this is a message writer discovery (QoS)
                        m_count += 1
                        m_bytes += len(raw(sub_msg))
                else:
                    # user_data
                    payload_size = len(raw(sub_msg.data))
                    msg_size = len(raw(sub_msg))
                    total_payload += payload_size
                    data_bytes += msg_size
                    data_count += 1


            elif isinstance(sub_msg, rtps.RTPSSubMessage_INFO_DST):
                # check DTS things
                pass
            elif isinstance(sub_msg, rtps.RTPSSubMessage_INFO_TS):
                if user_data:
                    # if the RPTS packet contains userdata
                    data_info_ts_bytes += len(raw(sub_msg))
                    data_info_ts_count += 1
                else:
                    discovery_info_ts_bytes += len(raw(sub_msg))
                    discovery_info_ts_count += 1
            else:
                # otherwise do nothing (ACKNACK, HEARTBEAT...)
                pass
        msg_total_count += 1
        msg_bytes_total += len(raw(pkt))
        if user_data:
            msg_data_count += 1
            msg_bytes_data += len(raw(pkt))
        else:
            msg_discovery_count += 1
            msg_bytes_discovery += len(raw(pkt))

    # summary table
    summary_tab = PrettyTable()
    summary_tab.field_names = [
            '# RTPS Messages ',
            'Bytes RTPS Messages',
        ]
    summary_tab.add_row([
        msg_total_count,
        msg_bytes_total,
    ])
    # discovery table
    discovery_tab = PrettyTable()
    discovery_tab.field_names = [
            '# Messages ',
            '# SubMessages ',
            'Bytes Total',
            'Bytes SubMessages',
            'Data(w) - (bytes)',
            'Data(r) - (bytes)',
            'Data(p) - (bytes)',
            'Data(m) - (bytes)',
            'Info_Ts - (bytes)',
        ]

    discovery_tab.add_row([
            msg_discovery_count,
            w_count+r_count+p_count+m_count+discovery_info_ts_count,
            msg_bytes_discovery,
            w_bytes+r_bytes+p_bytes+m_bytes+discovery_info_ts_bytes,
            f'{w_count} - ({w_bytes})',
            f'{r_count} - ({r_bytes})',
            f'{p_count} - ({p_bytes})',
            f'{m_count} - ({m_bytes})',
            f'{discovery_info_ts_count} - ({discovery_info_ts_bytes})',
        ])

    # data table
    data_tab = PrettyTable()
    data_tab.field_names = [
            '# Messages ',
            '# SubMessages ',
            'Bytes Total',
            'Bytes SubMessages',
            'Bytes Payload',
            'Data - (bytes)',
            'Info_Ts - (bytes)',
        ]

    data_tab.add_row([
            msg_data_count,
            data_count+data_info_ts_count,
            msg_bytes_data,
            data_bytes+data_info_ts_bytes,
            total_payload,
            f'{data_count} - ({data_bytes})',
            f'{data_info_ts_count} - ({data_info_ts_bytes})',
        ])

    return (summary_tab, discovery_tab, data_tab)

def filter_ts(ts_pkts):

    # TODO: we should measure also RTSP and ACKNAKs

    total_count = 0
    total_bytes = 0

    for pkt in ts_pkts:

        total_count += 1
        pkt_size = len(raw(pkt))
        total_bytes += pkt_size

    tab = PrettyTable()
    tab.field_names = [
            '# Messages ',
            'Bytes Total'
        ]

    tab.add_row([
            total_count,
            total_bytes
        ])

    return tab

def filter_user_data(data_pkts):

    # TODO: we should measure also RTSP and ACKNAKs

    total_count = 0
    total_payload = 0
    total_overhead = 0
    total_bytes = 0

    for pkt in data_pkts:
        # if the writerEntityKind is 0xC2 then we skip
        if pkt.writerEntityIdKind == 194:
            continue

        total_count += 1
        payload_size = len(raw(pkt.data))
        pkt_size = len(raw(pkt))
        overhead = pkt_size - payload_size

        total_payload += payload_size
        total_overhead += overhead
        total_bytes += pkt_size

    tab = PrettyTable()
    tab.field_names = [
            '# Messages ',
            'Bytes Total',
            'Bytes Payload',
            'Bytes Overhead',
        ]

    tab.add_row([
            total_count,
            total_bytes,
            total_payload,
            total_overhead,
        ])

    return tab


def main():
    parser = argparse.ArgumentParser(description='Parse DDS capture file')
    parser.add_argument('-d','--data', help='Pcap file path', required=True, type=str)

    args = vars(parser.parse_args())
    data = args['data']
    print(f'[ START ] Processing data in { data }')
    rtps_pkts = prepare(data)
    (summary_tab, discovery_tab, data_tab) = process(rtps_pkts)
    print(f"Summary:\n{summary_tab}")
    print(f"Discovery:\n{discovery_tab}")
    print(f"Data:\n{data_tab}")

    # print(f'[ RUN ] Found a total of {len(rtps_pkts)} RTPS packets')
    # data_rtps_pkts = filter_data(rtps_pkts)
    # ts_rtps_pkts = filter_info_ts(rtps_pkts)
    # print(f'[ RUN ] Found a total of {len(rtps_pkts)} RTPS DATA packets')
    # discovery_stats = filter_discovery(data_rtps_pkts)
    # print('Discovery:')
    # print(discovery_stats)
    # data_stats = filter_user_data(data_rtps_pkts)
    # print('Data:')
    # print(data_stats)
    # ts_statps = filter_ts(ts_rtps_pkts)
    # print('Timestamps:')
    # print(ts_statps)


if __name__ == '__main__':
    main()