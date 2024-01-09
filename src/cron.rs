use log::{info};

mod lib;
use lib::{
    version::print_version,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    print_version();

    info!("Starting cron");
}
