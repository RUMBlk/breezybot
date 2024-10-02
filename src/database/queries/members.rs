use sea_orm::prelude::Decimal;
use sea_orm::*;
use super::super::entities;
use entities::*;
use entities::prelude::*;
use std::future::Future;

pub async fn inselect(db: &DatabaseConnection, guild_id: &String, user_id: &String) -> Option<members::Model> {
    let _ = members::Entity::insert(
        members::ActiveModel {
            guild: Set(guild_id.clone()),
            user: Set(user_id.clone()),
            points: Set(0),
            ..Default::default()
        },
    )
    .on_conflict(sea_orm::sea_query::OnConflict::column(members::Column::Id).do_nothing().to_owned())
    .do_nothing()
    .exec(db)
    .await;

    Members::find()
        .filter(
            sea_orm::Condition::all()
                .add(members::Column::Guild.eq(guild_id))
                .add(members::Column::User.eq(user_id))
        )
        .one(db)
        .await.ok()?
}

pub fn server_value<'a>(db: &'a DatabaseConnection, guild_id: &'a String) -> impl Future<Output = Result<Option<Decimal>, DbErr>> + 'a {
    Members::find()
        .filter(members::Column::Guild.eq(guild_id))
        .select_only()
        .column_as(members::Column::Points.sum(), "sum")
        .into_tuple::<Decimal>()
        .one(db)
}

pub fn leaderboard(guild_id: &String, limit: Option<u64>) -> sea_orm::Select<entities::members::Entity> {
    Members::find()
        .filter(members::Column::Guild.eq(&guild_id.to_string()))
        .order_by_desc(members::Column::Points)
        .limit(limit.unwrap_or(10))
}