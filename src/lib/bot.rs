use std::collections::HashMap;

use teloxide::{prelude::*, utils::command::BotCommands};

use super::{
    domain::{verify_domain, DomainStatus},
    redis::{RedisActions, RedisClient},
};

pub fn get_bot() -> teloxide::Bot {
    Bot::from_env()
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum BotCommand {
    #[command(description = "display bot help")]
    Help,
    #[command(description = "add a domain that should be checked")]
    Add(String),
    #[command(description = "remove a domain from checklist")]
    Remove(String),
    #[command(description = "list all domains in your checklist")]
    List,
    #[command(description = "initiate checking domains manually")]
    Trigger,
    #[command(description = "manually check one single domain")]
    Check(String),
}

pub async fn message(
    bot: Bot,
    msg: Message,
    cmd: BotCommand,
    redis_client: RedisClient,
) -> ResponseResult<()> {
    // Store user for any command, so we know who's using the platform
    let _ = RedisActions::store_user(&redis_client, &msg.chat.id.to_string());

    match cmd {
        BotCommand::Help => {
            bot.send_message(msg.chat.id, BotCommand::descriptions().to_string())
                .await?
        }
        BotCommand::Add(domain) => {
            let _ = RedisActions::store_domain(&redis_client, &msg.chat.id.to_string(), &domain);
            bot.send_message(msg.chat.id, format!("Adding domain {domain}"))
                .await?
        }
        BotCommand::Remove(domain) => {
            match RedisActions::remove_domain(&redis_client, &msg.chat.id.to_string(), &domain) {
                Ok(e) => match e {
                    Some(_) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("{domain} was removed from your list"),
                        )
                        .await?
                    }
                    None => {
                        bot.send_message(
                            msg.chat.id,
                            format!("Domain {domain} was not found in your list"),
                        )
                        .await?
                    }
                },
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        format!("Something went wrong with removing {domain} from your list"),
                    )
                    .await?
                }
            }
        }
        BotCommand::List => {
            match RedisActions::get_domains(&redis_client, &msg.chat.id.to_string()) {
                Ok(domains) => match domains {
                    Some(domain_string) => {
                        let message = domain_string.split(",").collect::<Vec<&str>>().join("\n");
                        bot.send_message(
                            msg.chat.id,
                            format!("Current activated domains:\n\n{}", message),
                        )
                        .await?
                    }
                    None => {
                        bot.send_message(msg.chat.id, "No domains found for your user")
                            .await?
                    }
                },
                _ => {
                    bot.send_message(msg.chat.id, "No domains found for your user")
                        .await?
                }
            }
        }
        BotCommand::Trigger => {
            match RedisActions::get_domains(&redis_client, &msg.chat.id.to_string()) {
                Ok(domains) => match domains {
                    Some(domain_string) => {
                        let domains = domain_string.split(",").collect::<Vec<&str>>();
                        let mut domains_and_status: HashMap<&str, DomainStatus> = HashMap::new();

                        for domain in domains {
                            let status = verify_domain(domain);
                            domains_and_status.insert(domain, status);
                        }

                        let message = domains_and_status
                            .iter()
                            .map(|(domain, status)| match status {
                                DomainStatus::Expired => format!("{domain}: Certificate expired"),
                                DomainStatus::ValidationFailed(error) => {
                                    format!("{domain}: Check failed - {error}")
                                }
                                DomainStatus::ValidFor(days) => {
                                    format!("{domain}: Certificate valid for {days} days")
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n");

                        bot.send_message(msg.chat.id, message).await?
                    }
                    None => {
                        bot.send_message(msg.chat.id, "No domains found for your user")
                            .await?
                    }
                },
                _ => {
                    bot.send_message(msg.chat.id, "No domains found for your user")
                        .await?
                }
            }
        }
        BotCommand::Check(domain) => {
            let status = verify_domain(&domain);
            bot.send_message(msg.chat.id, format!("Checking domain {domain}"))
                .await?
        }
    };

    Ok(())
}
