# transaction processor

This is a simple transaction processor, that can process deposits, withdrawals, disputes, resolutions and chargebacks. It processes an input CSV file that contains transactions, and generates a CSV on stdout that contains the end status of all accounts.

## Code organization

* **amount**: Handles fixed-point amounts. The amounts are considered to have (up to) 4 decimals. To make it efficient without loss of precision nor conversions, the numeric value is stored as an i64, with a scaling value of 10000 (i.e., 1234 is represented as 12340000). It allows for basic arithmetic (addition and subtraction).
* **processor**: The main transaction processor code. It takes care of keeping the customer account data, as well as keeping track of disputes and a full transaction history.
* **csv_utils**: Handles the marshalling and unmarshalling of the CSV files.
* **types**: Data types used throughout the application.
* **main**: Main application entrypoint.

## Error handling

Error messages are sent to stderr, only if the `RUST_LOG` environment variable is set to `warn` or lower.

## Assumptions

* This has been tested with Rust 1.80.
* A dispute, resolve or chargeback for a particular transaction also has to match the client ID, so e.g. a client can't dispute another client's transaction.
* Any transactions for a locked account are ignored. There is currently no way to unlock a locked acount.
* Both disputed deposit and withdrawals will decrease the account's available funds and increase their held funds. This might not be correct.
* A dispute can result in a negative balance (the other option would be to ignore disputes that result in negative balances).

## Possible enhancements

* Other/multiple data sources. All the logic for processing transactions is in processor::Processor::process_txn. This should make it fairly easy to add other data sources. We would probably need to add multithreading and channels, at least for the ingestion.
* Limiting history. At the moment the transaction history for processing disputes is unbounded. If this becomes a problem we could set a limit to the number of transactions behind the current one that can be disputed. In that case we could use e.g. an [IndexMap](https://docs.rs/indexmap/latest/indexmap/map/struct.IndexMap.html) structure to store the history, and clean up old transactions periodically. If we still want it to be unbounded but lower memory usage, data from the evictd transactions that are evicted could be stored e.g. in a database.
