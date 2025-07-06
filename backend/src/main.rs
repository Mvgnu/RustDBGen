pub mod generated;

use generated::main::*;
use generated::router::*;

#[tokio::main]
async fn main() {
    if let Err(e) = generated::main::main().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
