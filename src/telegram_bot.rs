use teloxide::prelude::*;

mod lib;
use lib::{
    bot::{get_bot, message, BotCommand},
    redis::RedisClient,
};

#[tokio::main]
async fn main() {
    let redis_client = RedisClient::new();

    let bot = get_bot();
    BotCommand::repl(bot, move |bot, msg, cmd| {
        // redis service clone is required, otherwise we lose the reference
        message(bot, msg, cmd, redis_client.clone())
    })
    .await;
}
