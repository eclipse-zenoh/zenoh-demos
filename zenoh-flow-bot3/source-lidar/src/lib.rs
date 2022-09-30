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

use async_std::sync::{Arc, Mutex};
use async_trait::async_trait;
use hls_lfcd_lds_driver::{LFCDLaser, DEFAULT_BAUD_RATE, DEFAULT_PORT};
use std::{str::FromStr, time::Duration};
use zenoh_flow::prelude::*;
use zenoh_flow_example_types::ros2::tb3::LaserScan;

#[derive(Debug)]
struct Lidar;

impl Lidar {
    fn try_init_lfcd_laser(configuration: &Option<Configuration>) -> Result<Arc<Mutex<LFCDLaser>>> {
        let (port, baud_rate) = match configuration {
            Some(configuration) => {
                let port = match configuration["port"].as_str() {
                    Some(configured_port) => configured_port,
                    None => DEFAULT_PORT,
                };
                let baud_rate = match configuration["baudrate"].as_str() {
                    Some(configured_rate) => configured_rate,
                    None => DEFAULT_BAUD_RATE,
                };
                (port, baud_rate)
            }
            None => (DEFAULT_PORT, DEFAULT_BAUD_RATE),
        };

        let baud_rate = u32::from_str(baud_rate).map_err(|e| {
            zferror!(
                ErrorKind::ConfigurationError,
                "Unable to convert baud_rate: {}",
                e
            )
        })?;

        let lidar = LFCDLaser::new(port.to_string(), baud_rate)
            .map_err(|e| zferror!(ErrorKind::ConfigurationError, "Unable to open lidar: {}", e))?;

        Ok(Arc::new(Mutex::new(lidar)))
    }
}

#[async_trait]
impl Source for Lidar {
    async fn setup(
        &self,
        _context: &mut Context,
        configuration: &Option<Configuration>,
        mut outputs: Outputs,
    ) -> Result<Option<Box<dyn AsyncIteration>>> {
        let lidar = Lidar::try_init_lfcd_laser(configuration)?;
        let output = outputs.take_into_arc("Scan").unwrap();

        Ok(Some(Box::new(move || {
            let output = Arc::clone(&output);
            let lidar = Arc::clone(&lidar);

            async move {
                let data: Data;
                {
                    let mut lidar = lidar.lock().await;
                    let reading = lidar.read().await.map_err(|e| {
                        zferror!(ErrorKind::InvalidData, "Unable to read from lidar: {}", e)
                    })?;

                    data = Data::from(LaserScan(reading));
                }

                output.send_async(data, None).await?;
                async_std::task::sleep(Duration::from_millis(100)).await;
                Ok(())
            }
        })))
    }
}

zenoh_flow::export_source!(register);

fn register() -> Result<Arc<dyn Source>> {
    Ok(Arc::new(Lidar) as Arc<dyn Source>)
}
