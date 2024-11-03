pub mod lib;
pub mod activity;
pub mod elections;
pub mod localization;

use localization::loc;

use poise::serenity_prelude::{ GatewayIntents, ClientBuilder, Client, FullEvent as Event, GuildId };

use crate::Context;
use crate::Data;
use crate::Error;

#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let _ = ctx.defer().await;
    let _ = ctx.reply(format!("Ping: {}ms", ctx.ping().await.as_millis())).await;
    Ok(())
}

pub async fn build(data: Data) -> Option<Client> {
    let token = data.token.clone();
    let mut commands = vec![ping()];
    commands.extend(activity::commands());
    commands.extend(elections::commands());
    localization::apply_translation(&data.translations, &mut commands);
    let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
        commands: commands,
        event_handler: |ctx, event, _framework, data| {
            Box::pin(async move {
                match event {
                    Event::Ready { data_about_bot } => {
                        eprintln!("Successfully logged in as {}", data_about_bot.user.name);
                    },
                    Event::Message { new_message } => {
                        activity::on_message(ctx, data, new_message.clone()).await;
                        elections::on_message(ctx, data, new_message.clone()).await;
                    }
                    Event::ReactionAdd { add_reaction } => {
                        activity::on_reaction(ctx, data, add_reaction.clone()).await;
                    }
                    _ => {},
                };
                Ok(()) // Return Ok(()) to match the expected return type
            })
        },
        ..Default::default()
    })
    //.intents(GatewayIntents::all())
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            match data.mode.as_str() {
                "DEBUG" => { if let Some(debug_guild) = data.debug_guild { poise::builtins::register_in_guild(ctx, &framework.options().commands, GuildId::from(debug_guild)).await.expect(""); }},
                "PRODUCTION" => { poise::builtins::register_globally(ctx, &framework.options().commands).await?; },
                _ => {},
            }
            Ok(data)
        })
    })
    .build();

    ClientBuilder::new(token, GatewayIntents::all())
        .framework(framework)
        .await
        .ok()
}