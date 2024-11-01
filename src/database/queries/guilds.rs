use sea_orm::*;
use sea_query::OnConflict;
use super::super::entities;
use entities::*;
use entities::prelude::*;

pub async fn get(db: &DatabaseConnection, id: u64) -> Result<Option<guilds::Model>, DbErr> {
    Guilds::find_by_id(id).one(db).await
}

pub async fn insert(db: &DatabaseConnection, id: u64) -> Result<guilds::Model, DbErr> {
    guilds::ActiveModel {
        id: Set(id),
        locale: Set(String::from("en_US")),
        elections_channel: Set(None),
        ..Default::default()
    }.insert(db).await
}

pub async fn upsert_elections_channel(db: &DatabaseConnection, id: u64, channel_id: Option<u64>) -> Result<InsertResult<guilds::ActiveModel>, DbErr> {
    let model = guilds::ActiveModel {
        id: Set(id),
        elections_channel: Set(channel_id),
        ..Default::default()
    };

    Guilds::insert(model)
        .on_conflict(
            OnConflict::column(guilds::Column::Id)
                .update_column(guilds::Column::ElectionsChannel)
                .action_and_where(guilds::Column::ElectionsChannel.ne(channel_id))
                .to_owned()
        )
        .exec(db)
        .await
}

pub async fn locale(_db: &DatabaseConnection, _guild_id: &String) -> String {
    String::from("en_US")
}