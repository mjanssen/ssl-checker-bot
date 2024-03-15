use log::info;

use teloxide::prelude::*;

mod utils;
use utils::{
    bot::get_bot,
    redis::{RedisActions, RedisClient},
    version::print_version,
};

use crate::utils::domain::Checker;

#[tokio::main]
async fn main() {
    env_logger::init();
    print_version();

    info!("Starting cron");

    let redis_client = RedisClient::new();

    match RedisActions::get_clients_and_domains(&redis_client).await {
        Ok(clients_and_domains) => {
            let bot = get_bot();
            for (client, domains) in clients_and_domains {
                let domain_checker = Checker::new(domains, true);
                let domain_statusses = domain_checker.get_domain_statusses().await;

                let _ = bot
                    .send_message(
                        client,
                        domain_checker.parse_statusses_to_string(&domain_statusses),
                    )
                    .await;
            }
        }
        Err(_) => (),
    }
}
