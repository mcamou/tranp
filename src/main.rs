use std::io::stdout;

use log::{error, warn};

mod amount;
mod csv;
mod processor;
mod types;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        error!("Usage: {} <CSV file>", args[0]);
        std::process::exit(1);
    }

    let csv_path = &args[1];

    let txns = match csv::load(csv_path.to_string()) {
        Ok(data) => data,
        Err(e) => {
            error!("Error while reading CSV: {}", e);
            std::process::exit(1);
        }
    };

    let mut p = processor::Processor::new();

    for txn in txns {
        if let Err(e) = p.process_txn(&txn) {
            warn!("{}", e);
        }
    }

    if let Err(e) = csv::save(stdout(), p.get_accounts()) {
        error!("Error while writing CSV: {}", e);
        std::process::exit(1);
    }
}
