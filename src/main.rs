#![feature(alloc_system, proc_macro)]
extern crate alloc_system;

#[macro_use] extern crate log;
#[macro_use] extern crate error_chain;
extern crate env_logger;
extern crate rdkafka;
#[macro_use] extern crate serde_derive;
extern crate clap;

mod config;
mod error;
mod metadata;
mod scheduler;
mod utils;

use error::*;
use metadata::MetadataFetcher;
use config::Config;

use clap::{App, Arg, ArgMatches};

use std::time;
use std::thread;
use std::collections::HashMap;

extern crate serde_json;


// struct InstanceData {
//     config: Config,
//     metadata: MetadataFetcher,
// }
//
// impl InstanceData {
//     fn new(config: Config) -> InstanceData {
//         InstanceData {
//             config: config,
//             metadata: HashMap::new()
//         }
//     }
//
//     fn get_config(&self) -> &Config {
//         &self.config
//     }
// }


fn run_kafka_web(config_path: &str) -> Result<()> {
    // let mut instance_data = InstanceData::new(
    //         config::read_config(config_path)
    //         .chain_err(|| format!("Unable to load configuration from '{}'", config_path))?
    //     );
    // let config = instance_data.get_config();
    let config = config::read_config(config_path)
            .chain_err(|| format!("Unable to load configuration from '{}'", config_path))?;

    println!("CONFIG: {:?}", config);

    let mut fetcher = MetadataFetcher::new(time::Duration::from_secs(15));

    for (cluster_name, cluster_config) in config.clusters() {
        fetcher.add_cluster(cluster_name, &cluster_config.broker_string());
        info!("Added cluster {}", cluster_name);
    }

    loop {
        let cluster_id = "local_cluster".to_string();
        fetcher.get_metadata_copy(&cluster_id).map(|metadata| {
            let serialized = serde_json::to_string(&metadata).unwrap();
            info!("local cluster = {:?} {}", metadata.refresh_time(), serialized);
        });
        thread::sleep_ms(10000);
    }

    info!("Terminating");

    Ok(())
}

fn setup_args<'a>() -> ArgMatches<'a> {
    App::new("kafka web interface")
        .version(option_env!("CARGO_PKG_VERSION").unwrap_or(""))
        .about("Kafka web interface")
        .arg(Arg::with_name("conf")
             .short("c")
             .long("conf")
             .help("Configuration file")
             .takes_value(true)
             .required(true))
        .arg(Arg::with_name("log-conf")
             .long("log-conf")
             .help("Configure the logging format (example: 'rdkafka=trace')")
             .takes_value(true))
        .get_matches()
}

fn main() {
    let matches = setup_args();

    utils::setup_logger(true, matches.value_of("log-conf"));

    let config_path = matches.value_of("conf").unwrap();

    if let Err(ref e) = run_kafka_web(config_path) {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}
