use cdr::{CdrLe, Infinite};
use clap::{App, Arg};
use hls_lfcd_lds_driver::{LFCDLaser, LaserReading, DEFAULT_BAUD_RATE, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::f32::consts::PI;
use std::time::SystemTime;
use zenoh::config::Config;
use zenoh::prelude::r#async::AsyncResolve;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Time {
    sec: u32,
    nsec: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Header {
    stamp: Time,
    frame_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LaserScan {
    header: Header,
    angle_min: f32,
    angle_max: f32,
    angle_increment: f32,
    time_increment: f32,
    scan_time: f32,
    range_min: f32,
    range_max: f32,
    #[serde(with = "BigArray")]
    ranges: [f32; 360],
    #[serde(with = "BigArray")]
    intensities: [f32; 360],
}

impl From<LaserReading> for LaserScan {
    fn from(lr: LaserReading) -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        LaserScan {
            header: Header {
                stamp: Time {
                    sec: now.as_secs() as u32,
                    nsec: now.subsec_nanos() as u32,
                },
                frame_id: "laser".to_string(),
            },
            angle_increment: (2.0 * PI / 360.0),
            angle_min: 0.0,
            angle_max: 2.0 * PI - (2.0 * PI / 360.0),
            time_increment: 1.0 / (lr.rpms as f32 * 6.0),
            scan_time: 1.0 / (lr.rpms as f32 * 6.0) * 360.0,
            range_min: 0.12,
            range_max: 3.5,
            ranges: lr.ranges.map(|r| r as f32 / 1000.0),
            intensities: lr.intensities.map(|r| r as f32),
        }
    }
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let (config, key, port, baud_rate, delay) = parse_args();
    println!("Opening LDS01 on {} with {}", port, baud_rate);

    let mut port = LFCDLaser::new(port, baud_rate).unwrap();

    println!("Opening session...");
    let session = zenoh::open(config).res().await.unwrap();

    let publisher = session.declare_publisher(key).res().await.unwrap();
    loop {
        let laser_scan: LaserScan = port.read().await.unwrap().into();
        println!("Putting Data '{}': {:?}", publisher.key_expr(), laser_scan);
        publisher
            .put(cdr::serialize::<_, _, CdrLe>(&laser_scan, Infinite).unwrap())
            .res()
            .await
            .unwrap();
        async_std::task::sleep(std::time::Duration::from_millis(delay)).await;
    }
}

fn parse_args() -> (Config, String, String, u32, u64) {
    let args = App::new("zlidar")
        .about("zenoh turtlebot3 lidar demo")
        .arg(
            Arg::from_usage("-m, --mode=[MODE]  'The zenoh session mode (peer by default).")
                .possible_values(&["peer", "client"]),
        )
        .arg(Arg::from_usage(
            "-e, --connect=[LOCATOR]...   'Peer locators used to initiate the zenoh session.'",
        ))
        .arg(Arg::from_usage(
            "-l, --listener=[LOCATOR]...   'Locators to listen on.'",
        ))
        .arg(
            Arg::from_usage("-k, --key=[key_expr] 'The key expression to publish LaserReadings'")
                .default_value("rt/turtle1/lidar"),
        )
        .arg(Arg::from_usage("-p, --port=[port] 'The serial port.'").default_value(DEFAULT_PORT))
        .arg(
            Arg::from_usage("-b, --baud-rate=[baud-rate] 'The baud rate.'")
                .default_value(DEFAULT_BAUD_RATE),
        )
        .arg(
            Arg::from_usage("-d, --delay=[DELAY] 'The delay between each read in milliseconds.")
                .default_value("40"),
        )
        .arg(Arg::from_usage(
            "-c, --config=[FILE]      'A configuration file.'",
        ))
        .get_matches();

    let mut config = if let Some(conf_file) = args.value_of("config") {
        Config::from_file(conf_file).unwrap()
    } else {
        Config::default()
    };
    if let Some(Ok(mode)) = args.value_of("mode").map(|mode| mode.parse()) {
        config.set_mode(Some(mode)).unwrap();
    }
    if let Some(values) = args.values_of("connect") {
        config
            .connect
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }
    if let Some(values) = args.values_of("listen") {
        config
            .listen
            .endpoints
            .extend(values.map(|v| v.parse().unwrap()))
    }

    let key = args.value_of("key").unwrap().to_string();
    let port = args.value_of("port").unwrap().to_string();
    let baud_rate: u32 = args.value_of("baud-rate").unwrap().parse().unwrap();
    let delay: u64 = args.value_of("delay").unwrap().parse().unwrap();

    (config, key, port, baud_rate, delay)
}
