use crate::DB;
use crate::utils::misc::debug::UnwrapLog;
use crate::commands::setters::set_welcome_channel::WelcomeChannelData;
use crate::utils::{CommandResult, Context};

#[poise::command(
    prefix_command,
    slash_command,
    category = "Moderator",
    track_edits,
    required_permissions = "MANAGE_ROLES",
    guild_only,
    ephemeral
)]
pub async fn get_welcome_channel(
    ctx: Context<'_>,
) -> CommandResult {
    DB.use_ns("discord-namespace").use_db("discord").await?;

    let guild_id = ctx.guild_id().unwrap_log("No se pudo obtener el guild_id", file!(), line!())?;
    let sql_query = "SELECT * FROM welcome_channel WHERE guild_id = $guild_id";
    let existing_data: Option<WelcomeChannelData> = DB
        .query(sql_query)
        .bind(("guild_id", guild_id))
        .await?
        .take(0)?;

    if existing_data.is_none() {
        poise::say_reply(ctx, "No se ha establecido un canal de bienvenida").await?;
        return Ok(())
    }

    let result = existing_data.unwrap_log("No se encontró el canal de bienvenida", file!(), line!())?.channel_id;
    poise::say_reply(ctx, format!("El canal de bienvenida está establecido en <#{result}>")).await?;

    Ok(())
}