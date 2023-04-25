use std::env;
use serenity::{
    async_trait,
    model::{interactions::*, gateway::GatewayIntents},
    prelude::*,
};
use songbird::{SerenityInit, Songbird, input::ytdl};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            if command.data.name == "play" {
                let url = command.data.options[0].value.as_ref().unwrap().as_str().unwrap().to_string();
                println!("{}", url);
                let _ = command.create_interaction_response(&ctx.http, |response| {
                    response.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("Playing!"))
                }).await;

                let guild = command.guild_id.unwrap();
                let user_id = command.user.id;
                let member = guild.member(&ctx.http, user_id).await.unwrap();
                if let Some(channel) = member.voice_channel(&ctx.cache).await {
                    let manager = songbird::get(&ctx).await.unwrap().clone();
                    let (handler_lock, success) = manager.join(guild, channel).await;
                    if success.is_ok() {
                        let source = ytdl(&url).await.unwrap();
                        let handler = handler_lock.lock().await;
                        handler.play_source(source.into());
                    }
                } else {
                    let _ = command.edit_original_interaction_response(&ctx.http, |message| {
                        message.content("Necesitas estar en un canal de voz padre")
                    }).await;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let application_id: u64 = env::var("CLIENT_ID")
        .expect("Expected a client ID in the environment")
        .parse()
        .expect("CLIENT_ID must be a valid u64");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .intents(GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_VOICE_STATES)
        .application_id(application_id)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
