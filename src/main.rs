use teloxide::prelude::*;

use lib::{
    bot::{get_bot, message, BotCommand},
    redis::{RedisActions, RedisClient},
};
use ssl_expiration2::SslExpiration;

mod lib;

#[tokio::main]
async fn main() {
    let redis_client = RedisClient::new();
    let _ = RedisActions::store_user(&redis_client, "123123");

    let domain_names = ["ridecheck.app", "google.com", "oilpattern.com"];

    for domain_name in domain_names {
        match SslExpiration::from_domain_name(domain_name) {
            Ok(expiration) => {
                println!(
                    "{domain_name}: Cerfificate is still valid for {} days",
                    expiration.days()
                );
                if expiration.is_expired() {
                    println!("Certificate expired");
                }
            }
            _ => println!("Could not check domain ssl status"),
        }
    }

    let bot = get_bot();
    BotCommand::repl(bot, move |bot, msg, cmd| {
        // info!("Received command: Command::{:?}", cmd);
        // redis service clone is required, otherwise we lose the reference
        message(bot, msg, cmd, redis_client.clone())
    })
    .await;
}
