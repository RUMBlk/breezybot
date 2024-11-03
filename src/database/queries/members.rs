use sea_orm::{ *, prelude::{ Expr, Decimal } };
use super::super::entities;
use entities::*;
use entities::prelude::*;

pub async fn exists(db: &DatabaseConnection, guild: u64, user: u64) -> Option<i64> {
    Members::find()
        .filter(members::Column::Guild.eq(guild))
        .filter(members::Column::User.eq(user))
        .select_only()
        .column(members::Column::Id)
        .into_tuple()
        .one(db)
        .await
        .expect("")
}

pub async fn upsert(db: &DatabaseConnection, guild: u64, user: u64) -> Result<members::Model, DbErr> {
    let model = members::ActiveModel {
        guild: Set(guild.into()),
        user: Set(user.into()),
        ..Default::default()
    };
    model.update(db).await
}

pub async fn get(db: &DatabaseConnection, guild: u64, user: u64) -> Option<members::Model> {
    Members::find()
        .filter(members::Column::Guild.eq(guild))
        .filter(members::Column::User.eq(user))
        .one(db)
        .await
        .expect("")
}

pub async fn add_points(db: &DatabaseConnection, guild: u64, user: u64, points: i64) -> Result<InsertResult<members::ActiveModel>, DbErr> {
    Members::insert(members::ActiveModel {
        guild: Set(guild.into()),
        user: Set(user.into()),
        points: Set(points),
        ..Default::default()
    }).on_conflict(
        sea_query::OnConflict::columns(vec![members::Column::Guild, members::Column::User])
            .value(members::Column::Points, Expr::col((Members, members::Column::Points)).add(points))
            .to_owned()
    ).exec(db).await.inspect_err(|e| eprintln!("{e}"))
}

pub async fn server_value<'a>(db: &'a DatabaseConnection, guild: u64) -> Option<Decimal> {
    Members::find()
        .filter(members::Column::Guild.eq(guild))
        .select_only()
        .column_as(members::Column::Points.sum(), "sum")
        .into_tuple::<Decimal>()
        .one(db)
        .await
        .expect("")
}

pub async fn leaderboard(db: &DatabaseConnection, guild: u64, limit: Option<u64>) -> Result<Vec<members::Model>, DbErr> {
    Members::find()
        .filter(members::Column::Guild.eq(guild))
        .order_by_desc(members::Column::Points)
        .limit(limit.unwrap_or(10))
        .all(db)
        .await
        .inspect_err(|e| eprintln!("{e}"))
}