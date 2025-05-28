use cdr::{CdrLe, Infinite};
use clap::Parser;
use hls_lfcd_lds_driver::{LFCDLaser, LaserReading, DEFAULT_BAUD_RATE, DEFAULT_PORT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::f32::consts::PI;
use std::time::SystemTime;
use zenoh::{config::WhatAmI, Config, key_expr::KeyExpr};

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
    ranges: Vec<f32>,
    intensities: Vec<f32>,
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
            ranges: lr.ranges.map(|r| r as f32 / 1000.0).to_vec(),
            intensities: lr.intensities.map(|r| r as f32).to_vec(),
        }
    }
}

#[tokio::main]
async fn main() {
    zenoh::init_log_from_env_or("error");

    let (config, key, port, baud_rate, delay) = parse_args();
    println!("Opening LDS01 on {} with {}", port, baud_rate);

    let mut port = LFCDLaser::new(port, baud_rate).unwrap();

    println!("Opening session...");
    let session = zenoh::open(config).await.unwrap();

    let publisher = session.declare_publisher(key).await.unwrap();
    loop {
        let laser_scan: LaserScan = port.read().await.unwrap().into();
        println!("Putting Data '{}': {:?}", publisher.key_expr(), laser_scan);
        publisher
            .put(cdr::serialize::<_, _, CdrLe>(&laser_scan, Infinite).unwrap())
            
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
}

#[derive(clap::Parser, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Args {
    #[arg(short, long)]
    /// A configuration file.
    config: Option<String>,
    #[arg(long)]
    /// Allows arbitrary configuration changes as column-separated KEY:VALUE pairs, where:
    ///   - KEY must be a valid config path.
    ///   - VALUE must be a valid JSON5 string that can be deserialized to the expected type for the KEY field.
    ///
    /// Example: `--cfg='transport/unicast/max_links:2'`
    #[arg(long)]
    cfg: Vec<String>,
    #[arg(short, long)]
    /// The Zenoh session mode [default: peer].
    mode: Option<WhatAmI>,
    #[arg(short = 'e', long)]
    /// Endpoints to connect to.
    connect: Vec<String>,
    #[arg(short, long)]
    /// Endpoints to listen on.
    listen: Vec<String>,
    #[arg(long)]
    /// Disable the multicast-based scouting mechanism.
    no_multicast_scouting: bool,
    #[arg(short, long, default_value = "rt/turtle1/lidar")]
    /// The key expression to publish LaserReadings.
    key: KeyExpr<'static>,
    #[arg(short, long, default_value = DEFAULT_PORT)]
    /// The serial port.
    port: String,
    #[arg(short, long, default_value = DEFAULT_BAUD_RATE)]
    /// The baud rate.
    baud_rate: u32,
    #[arg(short, long, default_value = "40")]
    /// The delay between each read in milliseconds.
    delay: u64,
}

impl From<Args> for Config {
    fn from(value: Args) -> Self {
        (&value).into()
    }
}

impl From<&Args> for Config {
    fn from(args: &Args) -> Self {
        let mut config = match &args.config {
            Some(path) => Config::from_file(path).unwrap(),
            None => Config::default(),
        };
        if let Some(mode) = args.mode {
            config
                .insert_json5("mode", &json!(mode.to_str()).to_string())
                .unwrap();
        }

        if !args.connect.is_empty() {
            config
                .insert_json5("connect/endpoints", &json!(args.connect).to_string())
                .unwrap();
        }
        if !args.listen.is_empty() {
            config
                .insert_json5("listen/endpoints", &json!(args.listen).to_string())
                .unwrap();
        }
        if args.no_multicast_scouting {
            config
                .insert_json5("scouting/multicast/enabled", &json!(false).to_string())
                .unwrap();
        }
        for json in &args.cfg {
            if let Some((key, value)) = json.split_once(':') {
                if let Err(err) = config.insert_json5(key, value) {
                    eprintln!("`--cfg` argument: could not parse `{json}`: {err}");
                    std::process::exit(-1);
                }
            } else {
                eprintln!("`--cfg` argument: expected KEY:VALUE pair, got {json}");
                std::process::exit(-1);
            }
        }
        config
    }
}

fn parse_args() -> (Config, KeyExpr<'static>, String, u32, u64) {
    let args = Args::parse();
    ((&args).into(), args.key, args.port, args.baud_rate, args.delay)
}
