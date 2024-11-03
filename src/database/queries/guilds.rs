use sea_orm::{ *, prelude::Decimal };
use sea_query::OnConflict;
use super::super::entities;
use entities::*;
use entities::prelude::*;

pub async fn get(db: &DatabaseConnection, id: u64) -> Option<guilds::Model> {
    Guilds::find_by_id(id).one(db).await.expect("")
}

pub async fn insert(db: &DatabaseConnection, id: u64) -> Result<guilds::Model, DbErr> {
    guilds::ActiveModel {
        id: Set(id.into()),
        locale: Set(String::from("en_US")),
        elections_channel: Set(None),
        ..Default::default()
    }.insert(db).await
}

pub async fn upsert_elections_channel(db: &DatabaseConnection, id: u64, channel_id: Option<u64>) -> Result<InsertResult<guilds::ActiveModel>, DbErr> {
    let model = guilds::ActiveModel {
        id: Set(id.into()),
        elections_channel: Set(channel_id.map(|v| v.into())),
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
        .inspect_err(|e| eprintln!("{e}"))
}

pub async fn locale(db: &DatabaseConnection, id: u64) -> String {
    Guilds::find_by_id(id)
        .select_only()
        .column(guilds::Column::Locale)
        .into_tuple()
        .one(db)
        .await
        .expect("")
        .unwrap_or("en_US".to_owned())
}

pub async fn elections_channel(db: &DatabaseConnection, id: u64) -> Option<Decimal> {
    Guilds::find()
        .filter(guilds::Column::Id.eq(id))
        .select_only()
        .select_column(guilds::Column::ElectionsChannel)
        .into_tuple()
        .one(db)
        .await
        .expect("")
}