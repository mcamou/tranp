use std::io::stdout;

use itertools::sorted;
use log::{error, warn};

mod amount;
mod csv_utils;
mod processor;
mod types;

fn main() {
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        error!("Usage: {} <CSV file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];

    let mut p = processor::Processor::new();

    for e in p.process_csv(path.to_string()) {
        warn!("{}", e);
    }

    let accts = sorted(p.get_accounts());
    if let Err(e) = csv_utils::save(stdout(), accts) {
        error!("Error while writing CSV: {}", e);
        std::process::exit(1);
    }
}
