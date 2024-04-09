use sea_orm::*;
use super::super::entities;
use entities::*;
use entities::prelude::*;

pub async fn inselect(db: &DatabaseConnection, guild_id: &String) -> Option<guilds::Model> {
    let _ = guilds::Entity::insert(
        guilds::ActiveModel {
            guild: Set(guild_id.clone()),
            locale: Set(String::from("en_US")),
            elections_channel: Set(None),
            ..Default::default()
        },
    )
    .on_conflict(sea_orm::sea_query::OnConflict::column(guilds::Column::Id).do_nothing().to_owned())
    .do_nothing()
    .exec(db)
    .await;

    match Guilds::find()
    .filter(guilds::Column::Guild.eq(guild_id))
    .one(db)
    .await {
        Ok(member) => member,
        _ => None,
    }
}

pub async fn locale(_db: &DatabaseConnection, _guild_id: &String) -> String {
    String::from("en_US")
}