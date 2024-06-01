use serenity::all::Channel;

use crate::DB;
use crate::utils::{CommandResult, Context};
use crate::utils::config::{Channels, GuildData};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    required_permissions = "MODERATE_MEMBERS",
    guild_only,
    ephemeral
)]
pub async fn set_ooc_channel(
    ctx: Context<'_>,
    #[description = "The channel to set as the log channel"]
    ooc_channel: Channel,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;
    let guild_id = ctx.guild_id().unwrap();
    let channel_id = ooc_channel.id().to_string();

    let existing_data = GuildData::verify_data(guild_id).await?;
    if existing_data.is_none() {
        let data = GuildData::default()
            .guild_id(guild_id)
            .channels(Channels::default()
                .ooc(&channel_id)
            );
        data.save_to_db().await?;
        ctx.say(format!("OOC channel set to: <#{channel_id}>")).await?;

        return Ok(())
    }

    let data = Channels::default()
        .logs(&channel_id);

    data.update_field_in_db("channels.ooc", &channel_id, &guild_id.to_string()).await?;
    ctx.say(format!("Canal de Fuera de Contexto establecido en: <#{channel_id}>")).await?;

    Ok(())
}