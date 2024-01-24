use poise::Command;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, GuildId, MessageId, UserId};
use surrealdb::Result as SurrealResult;

pub mod autocomplete;
pub mod embeds;
pub mod handlers;
pub mod events;
pub mod debug;

use crate::DB;
use crate::commands::ping::ping;
use crate::commands::setters::set_admins::set_admins;
use crate::commands::setters::set_log_channel::set_log_channel;
use crate::commands::setters::set_timeout_role::set_time_out_role;
use crate::commands::setters::set_timeout_timer::set_timeout_timer;
use crate::commands::setters::set_forbidden_role::set_forbidden_role;
use crate::commands::setters::set_forbidden_user::set_forbidden_user;

use crate::commands::getters::get_admins::get_admins;
use crate::commands::getters::get_log_channel::get_log_channel;
use crate::commands::getters::get_timeout_role::get_timeout_role;
use crate::commands::getters::get_timeout_timer::get_timeout_timer;
use crate::commands::getters::get_forbidden_role::get_forbidden_role;
use crate::commands::getters::get_forbidden_user::get_forbidden_user;
use crate::commands::setters::set_timeout_message::set_time_out_message;
use crate::commands::setters::set_warn_message::set_warn_message;

pub struct Data {
    pub poise_mentions: String,
    pub client: reqwest::Client,
}

pub type CommandResult = Result<(), Error>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct MessageData {
    pub message_id: MessageId,
    pub message_content: String,
    pub author_id: UserId,
    pub channel_id: ChannelId,
    pub guild_id: Option<GuildId>,
}

impl MessageData {
    pub const fn new(
        message_id: MessageId,
        message_content: String,
        author_id: UserId,
        channel_id: ChannelId,
        guild_id: Option<GuildId>,
    ) -> Self {
        Self {
            message_id,
            message_content,
            author_id,
            channel_id,
            guild_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Warns {
    pub user_id: UserId,
    pub warns: u8,
}

impl Warns {
    pub const fn new(user_id: UserId) -> Self {
        Self { user_id, warns: 0 }
    }

    pub async fn get_warns(&self) -> SurrealResult<Option<Self>> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "SELECT * FROM warns WHERE user_id = $user_id";
        let existing_data: Option<Self> = DB
            .query(sql_query)
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        Ok(existing_data)
    }

    pub async fn save_to_db(&self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let _created: Vec<Self> = DB
            .create("warns")
            .content(self)
            .await?;

        println!("Created warns: {:?}", self.warns);

        Ok(())
    }

    pub async fn add_warn(&mut self) -> SurrealResult<()> {
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warns SET warns = $warns WHERE user_id = $user_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("warns", &self.warns))
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated warns: {:?}", self.warns);

        Ok(())
    }

    pub async fn reset_warns(&mut self) -> SurrealResult<()> {
        self.warns = 0;
        DB.use_ns("discord-namespace").use_db("discord").await?;
        let sql_query = "UPDATE warns SET warns = $warns WHERE user_id = $user_id";
        let _updated: Vec<Self> = DB
            .query(sql_query)
            .bind(("warns", &self.warns))
            .bind(("user_id", &self.user_id))
            .await?
            .take(0)?;

        println!("Updated warns: {:?}", self.warns);

        Ok(())
    }
}

pub fn load_commands() -> Vec<Command<Data, Error>> {
    vec![
        ping(),
        set_admins(),
        set_log_channel(),
        set_warn_message(),
        set_time_out_role(),
        set_timeout_timer(),
        set_forbidden_user(),
        set_forbidden_role(),
        set_time_out_message(),
        get_admins(),
        get_log_channel(),
        get_timeout_role(),
        get_timeout_timer(),
        get_forbidden_user(),
        get_forbidden_role(),
    ]
}