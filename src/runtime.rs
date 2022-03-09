use std::convert::TryFrom;

use std::fs::read_to_string;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::runtime::dataflow::loader::{Loader, LoaderConfig};
use zenoh_flow::runtime::RuntimeContext;

pub async fn runtime(name: String, descriptor_file: String) {
    env_logger::init();

    let yaml_df = read_to_string(descriptor_file).unwrap();

    let loader_config = LoaderConfig::new();

    let session = Arc::new(zenoh::open(zenoh::config::Config::default()).await.unwrap());
    let hlc = async_std::sync::Arc::new(uhlc::HLC::default());
    let loader = Arc::new(Loader::new(loader_config));

    let ctx = RuntimeContext {
        session,
        hlc,
        loader,
        runtime_name: name.clone().into(),
        runtime_uuid: uuid::Uuid::new_v4(),
    };

    // loading the descriptor
    let df =
        zenoh_flow::model::dataflow::descriptor::DataFlowDescriptor::from_yaml(&yaml_df).unwrap();

    // mapping to infrastructure
    let mapped = zenoh_flow::runtime::map_to_infrastructure(df, &name)
        .await
        .unwrap();

    // creating record
    let dfr =
        zenoh_flow::model::dataflow::record::DataFlowRecord::try_from((mapped, uuid::Uuid::nil()))
            .unwrap();

    // creating dataflow
    let dataflow = zenoh_flow::runtime::dataflow::Dataflow::try_new(ctx.clone(), dfr).unwrap();

    // instantiating
    let mut instance =
        zenoh_flow::runtime::dataflow::instance::DataflowInstance::try_instantiate(dataflow)
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

    let () = std::future::pending().await;
}
