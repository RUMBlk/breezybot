use sea_orm::*;
use crate::database as db;
use db::entities::*;
use db::entities::prelude::*;

pub async fn inselect(db: &DatabaseConnection, guild_id: String, role_id: String, col_limit: Option<i16>) -> Option<elections::Model> {
    match (
        elections::Entity::insert(
            elections::ActiveModel {
                guild: Set(guild_id.clone()),
                role: Set(role_id.clone()),
                limit: Set(col_limit.unwrap_or(1) as i16),
                ..Default::default()
            },
        )
        .on_conflict(sea_orm::sea_query::OnConflict::column(elections::Column::Id).do_nothing().to_owned())
        .do_nothing()
        .exec(db)
        .await,

        Elections::find()
        .filter(
            sea_orm::Condition::all()
                .add(elections::Column::Guild.eq(guild_id))
                .add(elections::Column::Role.eq(role_id))
        )
        .one(db)
        .await,
    ) {
        (Ok(_), Ok(row)) => row,
        _ => None,
    }
}