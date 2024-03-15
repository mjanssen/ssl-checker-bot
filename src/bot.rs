use log::info;
use teloxide::prelude::*;

mod utils;
use utils::{
    bot::{get_bot, message, BotCommand},
    redis::RedisClient,
    version::print_version,
};

#[tokio::main]
async fn main() {
    env_logger::init();
    print_version();

    info!("Starting bot service");

    let redis_client = RedisClient::new();

    let bot = get_bot();
    BotCommand::repl(bot, move |bot, msg, cmd| {
        // redis service clone is required, otherwise we lose the reference
        message(bot, msg, cmd, redis_client.clone())
    })
    .await;
}
