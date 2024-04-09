use sea_orm::prelude::Decimal;
use sea_orm::*;
use super::super::entities;
use entities::*;
use entities::prelude::*;

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

    match Members::find()
    .filter(
        sea_orm::Condition::all()
            .add(members::Column::Guild.eq(guild_id))
            .add(members::Column::User.eq(user_id))
    )
    .one(db)
    .await {
        Ok(member) => member,
        _ => None,
    }
}

pub async fn server_value(db: &DatabaseConnection, guild_id: &String) -> i64 {
    let res = Members::find()
    .filter(members::Column::Guild.eq(guild_id))
    .select_only()
    .column_as(members::Column::Points.sum(), "sum")
    .into_tuple::<Decimal>()
    .one(db)
    .await
    .unwrap()
    .unwrap();
    res.round().try_into().unwrap()
}