//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//

#define MAX_SAMPLES 1

#include <cstdlib>
#include <iostream>
#include <chrono>
#include <thread>
#include <argparse/argparse.hpp>

/* Include the C++ DDS API. */
extern "C" {
#include "dds/dds.h"
#include "Evaluation.h"
}


void ping(dds_entity_t participant, uint64_t msgs){

    uint32_t status = 0;
    void *samples[MAX_SAMPLES];
    dds_sample_info_t infos[MAX_SAMPLES];
    dds_return_t rc;
    dds_qos_t *qos;
    qos = dds_create_qos ();
    dds_qset_reliability(qos, DDS_RELIABILITY_RELIABLE, DDS_SECS (10));
    dds_qset_history(qos, DDS_HISTORY_KEEP_ALL, 10);

    // ping topic and writer initialization
    dds_entity_t ping_topic = dds_create_topic(participant, &Evaluation_Latency_desc, "ping", NULL, NULL);
    if (ping_topic < 0 ) exit(-1);

    dds_entity_t ping_writer = dds_create_writer(participant, ping_topic, qos, NULL);
    if ( ping_writer < 0 )  exit(-1);

    // pong topic and subscriber initialization

    dds_entity_t pong_topic = dds_create_topic(participant, &Evaluation_Latency_desc, "pong", NULL, NULL);
    if (pong_topic < 0 ) exit(-1);

    dds_entity_t pong_reader = dds_create_reader(participant, pong_topic, qos, NULL);
    if ( pong_reader < 0 )  exit(-1);

    auto pace = std::chrono::duration<double>(1.0/double(msgs));

    samples[0] = Evaluation_Latency__alloc ();


    rc = dds_set_status_mask(ping_writer, DDS_PUBLICATION_MATCHED_STATUS);
    if (rc != DDS_RETCODE_OK) DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));

    while(!(status & DDS_PUBLICATION_MATCHED_STATUS))
    {
        rc = dds_get_status_changes (ping_writer, &status);
        if (rc != DDS_RETCODE_OK)
        DDS_FATAL("dds_get_status_changes: %s\n", dds_strretcode(-rc));

        /* Polling sleep. */
        dds_sleepfor (DDS_MSECS (20));
    }

    while (true) {
        Evaluation_Latency msg;

        std::this_thread::sleep_for(pace);
        auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch());
        msg.ts = ts.count();

        rc = dds_write(ping_writer, &msg);
        if (rc < 0 ) DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));

        rc = dds_take(pong_reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
        while ( rc <= 0) {
            rc = dds_take(pong_reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
            if (rc < 0 ) DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));
        }

    }
}


typedef struct {
    dds_entity_t writer;
    uint64_t msgs;
} pp_args;


void ping_cb(dds_entity_t reader, void *args) {
    dds_return_t rc;
    void *samples[MAX_SAMPLES];
    dds_sample_info_t infos[MAX_SAMPLES];
    samples[0] = Evaluation_Latency__alloc ();

    Evaluation_Latency msg;
    Evaluation_Latency *rmsg;

    auto ts = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::system_clock::now().time_since_epoch()).count();

    pp_args* writer_struct = (pp_args*) args;


    rc = dds_take(reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
    while ( rc <= 0) {
        rc = dds_take(reader, samples, infos, MAX_SAMPLES, MAX_SAMPLES);
        if (rc < 0 ) DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));
    }

    rmsg = (Evaluation_Latency*) samples[0];
    auto latency = ts - rmsg->ts;
    std::cout << "cyclonedds,scenario,latency,pipeline," << writer_struct->msgs << ",1," << latency << ",us" << std::endl << std::flush;
    msg.ts = 0;
    rc = dds_write(writer_struct->writer, &msg);
    if (rc < 0 ) DDS_FATAL("dds_read: %s\n", dds_strretcode(-rc));

    Evaluation_Latency_free (samples[0], DDS_FREE_ALL);

}



void pong(dds_entity_t participant, uint64_t msgs){

    uint32_t status = 0;
    dds_listener_t *listeners;
    dds_qos_t *qos;
    dds_return_t rc;
    qos = dds_create_qos ();
    dds_qset_reliability(qos, DDS_RELIABILITY_RELIABLE, DDS_SECS (10));
    dds_qset_history(qos, DDS_HISTORY_KEEP_ALL, 10);

    listeners = dds_create_listener(NULL);

    // ping topic and writer initialization
    dds_entity_t ping_topic = dds_create_topic(participant, &Evaluation_Latency_desc, "ping", NULL, NULL);
    if (ping_topic < 0 ) exit(-1);


    // pong topic and subscriber initialization

    dds_entity_t pong_topic = dds_create_topic(participant, &Evaluation_Latency_desc, "pong", NULL, NULL);
    if (pong_topic < 0 ) exit(-1);

    dds_entity_t pong_writer = dds_create_writer(participant, pong_topic, qos, NULL);
    if ( pong_writer < 0 )  exit(-1);

    pp_args writer_struct;
    writer_struct.writer = pong_writer;
    writer_struct.msgs = msgs;

    rc = dds_set_status_mask(pong_writer, DDS_PUBLICATION_MATCHED_STATUS);
    if (rc != DDS_RETCODE_OK) DDS_FATAL("dds_set_status_mask: %s\n", dds_strretcode(-rc));

    while(!(status & DDS_PUBLICATION_MATCHED_STATUS))
    {
        rc = dds_get_status_changes (pong_writer, &status);
        if (rc != DDS_RETCODE_OK)
        DDS_FATAL("dds_get_status_changes: %s\n", dds_strretcode(-rc));

        /* Polling sleep. */
        dds_sleepfor (DDS_MSECS (20));
    }

    dds_lset_data_available_arg(listeners, ping_cb, &writer_struct, false);

    dds_entity_t ping_reader = dds_create_reader(participant, ping_topic, qos, listeners);
    if ( ping_reader < 0 )  exit(-1);





    while (true) {
        std::this_thread::sleep_for(std::chrono::duration<int>(10));
    }

}

int main(int argc, char* argv[]){

    // Argument parsing
    argparse::ArgumentParser program("ping-pong-cyclone");

    program.add_argument("msgs")
        .help("messages per second")
        .scan<'i', uint64_t>();

    program.add_argument("--ping")
        .help("is the ping")
        .default_value(false)
        .implicit_value(true);



    try {
        program.parse_args(argc, argv);
    }
    catch (const std::runtime_error& err) {
        std::cerr << err.what() << std::endl;
        std::cerr << program;
        std::exit(1);
    }

    uint64_t msgs = program.get<uint64_t>("msgs");
    bool ping_flag = false;

    if (program["--ping"] == true) {
        ping_flag = true;
    }

    // DDS initialization

    dds_entity_t participant = dds_create_participant (DDS_DOMAIN_DEFAULT, NULL, NULL);
    if ( participant < 0 ) exit(-1);

    if (ping_flag) {
        ping(participant, msgs);
    } else {
        pong(participant, msgs);
    }


}