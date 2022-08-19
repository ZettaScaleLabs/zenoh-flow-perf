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

use std::convert::TryFrom;
use std::fs::read_to_string;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;

pub enum Descriptor {
    Composed(String),
    Flatten(String),
}

fn _write_record_to_file(
    record: zenoh_flow::model::dataflow::record::DataFlowRecord,
    filename: &str,
) {
    let path = Path::new(filename);
    let mut write_file = File::create(path).unwrap();
    write!(write_file, "{}", record.to_yaml().unwrap()).unwrap();
}

pub async fn runtime(
    name: String,
    descriptor: Descriptor,
    listen: Vec<String>,
    connect: Vec<String>,
) {
    env_logger::init();

    let loader_config = LoaderConfig::new();

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();

    for l in listen {
        config.listen.endpoints.push(l.parse().unwrap());
    }
    for c in connect {
        config.connect.endpoints.push(c.parse().unwrap());
    }

    let session = Arc::new(zenoh::open(config).await.unwrap());
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let loader = Arc::new(Loader::new(loader_config));

    let ctx = RuntimeContext {
        session,
        hlc,
        loader,
        runtime_name: name.clone().into(),
        runtime_uuid: uuid::Uuid::new_v4(),
    };

    // loading the descriptor, it can be a flattened one, or a composed one
    let df = match descriptor {
        Descriptor::Composed(descriptor_file) => {
            let yaml_df = read_to_string(descriptor_file).unwrap();
            let df =
                zenoh_flow::model::dataflow::descriptor::DataFlowDescriptor::from_yaml(&yaml_df)
                    .unwrap();
            df.flatten().await.unwrap()
        }
        Descriptor::Flatten(descriptor_file) => {
            let yaml_df = read_to_string(descriptor_file).unwrap();
            zenoh_flow::model::dataflow::descriptor::FlattenDataFlowDescriptor::from_yaml(&yaml_df)
                .unwrap()
        }
    };

    // mapping to infrastructure
    let mapped = zenoh_flow::runtime::map_to_infrastructure(df, &name)
        .await
        .unwrap();

    // creating record
    let dfr =
        zenoh_flow::model::dataflow::record::DataFlowRecord::try_from((mapped, uuid::Uuid::nil()))
            .unwrap();

    _write_record_to_file(dfr.clone(), &format!("{}-record.yaml", dfr.flow));

    // creating dataflow
    let dataflow = zenoh_flow::runtime::dataflow::Dataflow::try_new(ctx.clone(), dfr).unwrap();

    // instantiating
    let mut instance = zenoh_flow::runtime::dataflow::instance::DataflowInstance::try_instantiate(
        dataflow,
        ctx.hlc.clone(),
    )
    .unwrap();

    let mut sinks = instance.get_sinks();
    for id in sinks.drain(..) {
        instance.start_node(&id).await.unwrap()
    }

    let mut operators = instance.get_operators();
    for id in operators.drain(..) {
        instance.start_node(&id).await.unwrap()
    }

    let mut connectors = instance.get_connectors();
    for id in connectors.drain(..) {
        instance.start_node(&id).await.unwrap()
    }

    let sources = instance.get_sources();
    for id in &sources {
        instance.start_node(id).await.unwrap()
    }

    std::future::pending::<()>().await;
}
