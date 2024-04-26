use std::sync::Arc;
use serenity::all::{GuildId, Message, UserId};
use poise::serenity_prelude as serenity;
use crate::DB;
use crate::utils::CommandResult;
use crate::utils::MessageData;
use crate::commands::setters::{AdminData, ForbiddenUserData, SetTimeoutTimer};
use crate::commands::setters::ForbiddenRoleData;
use crate::utils::misc::debug::UnwrapLog;
use crate::utils::handlers::misc::attachment_case::attachment_handler;
use crate::utils::handlers::misc::everyone_case::handle_everyone;
use crate::utils::handlers::misc::forbidden_mentions::{handle_forbidden_role, handle_forbidden_user};
use crate::utils::handlers::misc::link_spam_handler::{extract_link, spam_checker};

const CURRENT_MODULE: &str = file!();

/// # Esta función maneja los mensajes enviados en un servidor
///
/// ## Funciones relacionadas:
/// - manejo de archivos adjuntos
/// - manejo de menciones a roles y usuarios prohibidos
/// - manejo de menciones a @everyone y @here
/// - manejo de spam de links
/// - guardar el mensaje en la base de datos
pub async fn message_handler(ctx: &serenity::Context, new_message: &Message) -> CommandResult {

    // Crear un objeto Arc<String> con el contenido del mensaje para utilizar cheap cloning (clonación barata)
    // La clonación barata consiste en utilizar Arc<T> o Rc<T> para clonar un objeto sin copiar su contenido
    // Rc<T> es para usar en hilos de ejecución y Arc<T> es para usar en hilos de ejecución concurrentes (async)
    let message_content = Arc::new(String::from(&new_message.content));
    if new_message.author.bot { return Ok(()) }
    let guild_id = new_message.guild_id.unwrap_log("No se pudo obtener el id del servidor", CURRENT_MODULE, line!())?;
    let mut member = guild_id.member(&ctx.http, new_message.author.id).await?;
    let user_id = new_message.mentions.first().map(|user| user.id);
    let admin_role_id = AdminData::get_admin_role(guild_id).await?;

    // Mover a la función de handle_warns y handle_everyone?
    let time_out_timer = SetTimeoutTimer::get_time_out_timer(guild_id).await?;
    let time = time_out_timer.unwrap_or_default(); // SAFETY: Si se establece en 0, es porque no se ha establecido un tiempo de silencio

    // Si hay un error al manejar un archivo adjunto, imprimir el error pero no terminar la función
    if let Err(why) = attachment_handler(new_message).await {
        println!("Error handling attachment: {why:?} {CURRENT_MODULE} : {}", line!());
    }

    let data = MessageData::new(
        new_message.id,
        &message_content,
        new_message.author.id,
        new_message.channel_id,
        new_message.guild_id,
    );

    // Extraer el link del mensaje si existe
    if extract_link(&message_content).is_some() {
        let channel_id = new_message.channel_id;
        spam_checker(&message_content, channel_id, &admin_role_id, ctx, time, new_message, guild_id).await?;
    }

    if user_id.is_some() {
        handle_user_id(ctx, new_message, guild_id, &data, user_id).await?;
    }

    // @everyone no tiene id, por lo que no es necesario el <@id>
    if message_content.contains("@everyone") || message_content.contains("@here") {
        let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;
        handle_everyone(admin_role_id, &mut member, ctx, time, new_message).await?;

        return Ok(())
    }

    let _created: Vec<MessageData> = DB.create("messages").content(&data).await?;

    Ok(())
}

/// # Esta función maneja las menciones a usuarios y roles prohibidos
///
/// ## Funciones relacionadas:
/// - manejo de menciones a usuarios prohibidos
/// - manejo de menciones a roles prohibidos
/// - silenciar al autor del mensaje
/// - guardar el mensaje en la base de datos
async fn handle_user_id(
    ctx: &serenity::Context,
    new_message: &Message,
    guild_id: GuildId,
    data: &MessageData,
    user_id: Option<UserId>
) -> CommandResult {
    // Si el mensaje contiene una mención a un usuario prohibido, silenciar al autor del mensaje
    let forbidden_user_id = ForbiddenUserData::get_forbidden_user_id(guild_id).await?;
    if let Some(forbidden_user_id_some) = forbidden_user_id {
        if new_message.mentions_user_id(forbidden_user_id_some) {
            handle_forbidden_user(ctx, new_message, guild_id, data, forbidden_user_id_some).await?;
            let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
            return Ok(())
        }
    }

    // Si el mensaje contiene una mención a un rol prohibido, silenciar al autor del mensaje
    let database_data = ForbiddenRoleData::get_role_id(guild_id).await?;
    let forbidden_role_id = database_data.unwrap_log("No se ha establecido un rol prohibido de mencionar", CURRENT_MODULE, line!())?;
    let has_role = user_id
        .unwrap()
        .to_user(&ctx.http).await?
        .has_role(&ctx.http, guild_id, forbidden_role_id).await?;

    if has_role {
        handle_forbidden_role(ctx, new_message, guild_id, data).await?;
        let _created: Vec<MessageData> = DB.create("messages").content(data).await?;
    };

    Ok(())
}