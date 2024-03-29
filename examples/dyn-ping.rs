//
// Copyright (c) 2017, 2022 ZettaScale Technology.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale zenoh team, <zenoh@zettascale.tech>
//

use std::sync::Arc;
use zenoh_flow::prelude::*;
use zenoh_flow_perf::nodes::PingSourceFactory;

export_source_factory!(register);

fn register() -> Result<Arc<dyn SourceFactoryTrait>> {
    Ok(Arc::new(PingSourceFactory) as Arc<dyn SourceFactoryTrait>)
}
