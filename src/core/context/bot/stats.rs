use crate::core::BotContext;
use chrono::{DateTime, Utc};
use git_version::git_version;
use std::sync::atomic::{AtomicUsize, Ordering};
use twilight::model::channel::Message;

#[derive(Debug)]
pub struct BotStats {
    pub start_time: DateTime<Utc>,
    pub user_messages: AtomicUsize,
    pub bot_messages: AtomicUsize,
    pub my_messages: AtomicUsize,
    pub error_count: AtomicUsize,
    pub commands_ran: AtomicUsize,
    pub custom_commands_ran: AtomicUsize,
    pub guilds: AtomicUsize,
    pub version: &'static str,
}

impl BotStats {
    pub async fn new_message(&self, ctx: &BotContext, msg: &Message) {
        if msg.author.bot {
            // This will simply skip incrementing it if we couldn't get
            // a lock on the cache. No harm done.
            if ctx.is_own(msg) {
                ctx.stats.my_messages.fetch_add(1, Ordering::Relaxed);
            }
            ctx.stats.bot_messages.fetch_add(1, Ordering::Relaxed);
        } else {
            ctx.stats.user_messages.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub async fn had_error(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn new_guild(&self) {
        self.guilds.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn left_guild(&self) {
        self.guilds.fetch_sub(1, Ordering::Relaxed);
    }

    pub async fn command_used(&self, is_custom: bool) {
        if !is_custom {
            self.commands_ran.fetch_add(1, Ordering::Relaxed);
        } else {
            self.custom_commands_ran.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Default for BotStats {
    fn default() -> Self {
        BotStats {
            start_time: Utc::now(),
            user_messages: AtomicUsize::new(0),
            bot_messages: AtomicUsize::new(0),
            my_messages: AtomicUsize::new(0),
            error_count: AtomicUsize::new(0),
            commands_ran: AtomicUsize::new(0),
            custom_commands_ran: AtomicUsize::new(0),
            guilds: AtomicUsize::new(0),
            version: git_version!(),
        }
    }
}

#[derive(Debug)]
pub struct LoadingState {
    to_load: u32,
    loaded: u32,
}