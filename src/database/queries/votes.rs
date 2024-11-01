use sea_orm::{ 
    prelude::{Decimal, Expr}, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect
};

use super::super::entities;
use entities::{ *, prelude::* };

pub async fn sum_votes(db: &DatabaseConnection, role: u64) -> Result<Vec<String>, sea_orm::DbErr> {
    let limit = Elections::find_by_id(role)
        .select_only()
        .column(elections::Column::Limit)
        .into_tuple()
        .one(db)
        .await?;
    
    votes::Entity::find()
        .inner_join(Members)
        .inner_join(Candidates)
        .filter(candidates::Column::Elections.eq(role))
        .filter(
            candidates::Column::BannedUntil.is_null()
                .or(Expr::current_timestamp().gte(Expr::col(candidates::Column::BannedUntil)))
        )
        .group_by(candidates::Column::Id)
        .order_by_desc(members::Column::Points.sum())
        .select_only()
        .column(candidates::Column::User)
        .limit(limit)
        .into_tuple::<String>()
        .all(db)
        .await
}

pub async fn leaderboard(db: &DatabaseConnection, role: u64, limit: u64) -> Result<Vec<(u64, sea_orm::prelude::Decimal)>, sea_orm::DbErr> {
    Votes::find()
        .inner_join(Members)
        .inner_join(Candidates)
        .filter(candidates::Column::Elections.eq(role))
        .filter(
            candidates::Column::BannedUntil.is_null()
                .or(Expr::current_timestamp().gte(Expr::col(candidates::Column::BannedUntil)))
        )
        .group_by(candidates::Column::Id)
        .order_by_desc(members::Column::Points.sum())
        .select_only()
        .column(candidates::Column::User)
        .column_as(members::Column::Points.sum(), "points")
        .limit(limit)
        .into_tuple::<(u64, sea_orm::prelude::Decimal)>()
        .all(db)
        .await
}

pub async fn points_total(db: &DatabaseConnection, role: u64) -> Result<Option<Decimal>, DbErr> {
    Votes::find()
        .inner_join(members::Entity)
        .inner_join(candidates::Entity)
        .filter(candidates::Column::Elections.eq(role))
        .select_only()
        .column_as(members::Column::Points.sum(), "total")
        .into_tuple::<sea_orm::prelude::Decimal>()
        .one(db)
        .await
}

pub async fn affected_elections_by_user(db: &DatabaseConnection, user: u64) -> Result<Vec<u64>, sea_orm::DbErr> {
    Votes::find()
        .inner_join(members::Entity)
        .inner_join(candidates::Entity)
        .filter(members::Column::User.eq(user))
        .group_by(candidates::Column::Elections)
        .select_only()
        .column(candidates::Column::Elections)
        .into_tuple::<u64>()
        .all(db)
        .await
}