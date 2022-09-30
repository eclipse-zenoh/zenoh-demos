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

use async_trait::async_trait;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};
use zenoh_flow::prelude::*;
use zenoh_flow_example_types::ZFUsize;

#[derive(Debug)]
struct Tick;

#[async_trait]
impl Source for Tick {
    async fn setup(
        &self,
        _context: &mut Context,
        _configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let count = Arc::new(AtomicUsize::new(0));
        let output = outputs.take_into_arc("Tick").unwrap();

        Ok(Some(Box::new(move || {
            let c_output = Arc::clone(&output);
            let c_count = Arc::clone(&count);
            async move {
                c_count.fetch_add(1, Ordering::AcqRel);
                let data = Data::from(ZFUsize(c_count.load(Ordering::Relaxed)));
                c_output.send_async(data, None).await?;
                async_std::task::sleep(Duration::from_millis(100)).await;
                Ok(())
            }
        })))
    }
}

zenoh_flow::export_source!(register);

fn register() -> Result<Arc<dyn Source>> {
    Ok(Arc::new(Tick) as Arc<dyn Source>)
}
