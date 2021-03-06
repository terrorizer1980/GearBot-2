use chrono::{DateTime, Utc};
use twilight_gateway::Cluster;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::Message,
    id::{GuildId, UserId},
    user::CurrentUser,
};

mod cold_resume;
mod data_access;
mod logpump;
mod permissions;
mod stats;

pub mod status;

pub use stats::BotStats;

use crate::cache::Cache;
use crate::core::logpump::LogData;
use crate::core::GuildConfig;
use crate::database::api_structs::{RawTeamMembers, TeamInfo, TeamMember};
use crate::database::DataStorage;
use crate::translation::{GearBotString, Translations};
use crate::SchemeInfo;
use fluent_bundle::FluentArgs;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;
use unic_langid::LanguageIdentifier;

#[derive(PartialEq, Debug)]
pub enum ShardState {
    PendingCreation,
    Connecting,
    Identifying,
    Connected,
    Ready,
    Resuming,
    Reconnecting,
    Disconnected,
}

pub struct BotContext {
    pub cache: Cache,
    pub cluster: Cluster,
    pub http: HttpClient,
    pub stats: Arc<BotStats>,
    pub status_type: RwLock<u16>,
    pub status_text: RwLock<String>,
    pub bot_user: CurrentUser,
    configs: RwLock<HashMap<GuildId, Arc<GuildConfig>>>,
    pub datastore: DataStorage,
    pub translations: Translations,
    pub scheme_info: SchemeInfo,
    pub shard_states: RwLock<HashMap<u64, ShardState>>,
    pub start_time: DateTime<Utc>,
    pub global_admins: Vec<UserId>,
    team_info: RawTeamMembers,
    logpump_sender: UnboundedSender<LogData>,
}

impl BotContext {
    pub fn new(
        bot_core: (Cache, Cluster, SchemeInfo),
        http_info: (HttpClient, CurrentUser),
        datastore: DataStorage,
        translations: Translations,
        global_admins: Vec<u64>,
        stats: Arc<BotStats>,
        logpump_sender: UnboundedSender<LogData>,
    ) -> Self {
        let scheme_info = bot_core.2;
        let mut shard_states = HashMap::with_capacity(scheme_info.shards_per_cluster as usize);
        for i in scheme_info.cluster_id * scheme_info.shards_per_cluster
            ..scheme_info.cluster_id * scheme_info.shards_per_cluster + scheme_info.shards_per_cluster
        {
            shard_states.insert(i, ShardState::PendingCreation);
            bot_core
                .0
                .missing_per_shard
                .write()
                .expect("Global shard state tracking got poisoned!")
                .insert(i, AtomicU64::new(0));
        }

        let global_admins = global_admins.into_iter().map(UserId).collect();

        stats.shard_counts.pending.set(scheme_info.shards_per_cluster as i64);

        let team_info: RawTeamMembers =
            toml::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/team.toml"))).unwrap();

        BotContext {
            cache: bot_core.0,
            cluster: bot_core.1,
            http: http_info.0,
            stats,
            status_type: RwLock::new(3),
            status_text: RwLock::new(String::from("the commands turn")),
            bot_user: http_info.1,
            configs: RwLock::new(HashMap::new()),
            datastore,
            translations,
            scheme_info,
            shard_states: RwLock::new(shard_states),
            start_time: Utc::now(),
            global_admins,
            team_info,
            logpump_sender,
        }
    }

    /// Returns if a message was sent by us.
    pub fn is_own(&self, other: &Message) -> bool {
        self.bot_user.id == other.author.id
    }

    pub fn translate(&self, language: &LanguageIdentifier, key: GearBotString) -> String {
        self.translations.get_text_plain(language, key).to_string()
    }

    pub fn translate_with_args(
        &self,
        language: &LanguageIdentifier,
        string_key: GearBotString,
        args: &FluentArgs<'_>,
    ) -> String {
        self.translations
            .get_text_with_args(language, string_key, args)
            .replace("\\n", "\n")
    }

    pub async fn get_team_info(&self) -> TeamInfo {
        let mut members = vec![];
        for m in &self.team_info.members {
            let user = self.get_user(UserId(m.id.parse().unwrap())).await.unwrap();
            members.push(TeamMember {
                username: user.username.clone(),
                discriminator: user.discriminator.clone(),
                id: m.id.to_string(),
                avatar: user.avatar.clone().unwrap_or_default(),
                team: m.team.clone(),
                socials: m.socials.clone(),
            });
        }

        TeamInfo { members }
    }

    pub fn log(&self, data: LogData) {
        // can only error if the other side is closed, and we never close the main receiver
        let _ = self.logpump_sender.send(data);
        self.stats.logpump_stats.pending_logs.inc();
    }
}
