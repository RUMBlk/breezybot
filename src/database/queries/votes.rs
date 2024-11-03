use sea_orm::{ 
    prelude::{Decimal, Expr}, sea_query::Alias, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait
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
        .filter(candidates::Column::Active.eq(true))
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
        .filter(candidates::Column::Active.eq(true))
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

pub async fn points_total(db: &DatabaseConnection, role: u64) -> Option<Decimal> {
    Votes::find()
        .inner_join(Members)
        .inner_join(Candidates)
        .filter(candidates::Column::Elections.eq(role))
        .filter(
            candidates::Column::BannedUntil.is_null()
                .or(Expr::current_timestamp().gte(Expr::col(candidates::Column::BannedUntil)))
        )
        .filter(candidates::Column::Active.eq(true))
        .select_only()
        .column_as(members::Column::Points.sum(), "total")
        .into_tuple::<sea_orm::prelude::Decimal>()
        .one(db)
        .await
        .expect("")
}

pub async fn affected_elections_by_user(db: &DatabaseConnection, guild: u64, user: u64) -> Result<Vec<u64>, sea_orm::DbErr> {
    Votes::find()
        .inner_join(Members)
        .inner_join(Candidates)
        .join_as(
            sea_orm::JoinType::InnerJoin,
            candidates::Relation::Elections.def(),
            Alias::new("elections"),
        )
        .filter(members::Column::User.eq(user))
        .filter(elections::Column::Guild.eq(guild))
        .group_by(candidates::Column::Elections)
        .select_only()
        .column(candidates::Column::Elections)
        .into_tuple::<u64>()
        .all(db)
        .await
}

pub async fn remove_candidate_votes(db: &DatabaseConnection, id: i64) -> Result<sea_orm::DeleteResult, DbErr> {
    Votes::delete_many().filter(votes::Column::Id.eq(id)).exec(db).await
}

pub async fn cast(db: &DatabaseConnection, member: i64, candidate: i64) -> Result<sea_orm::InsertResult<votes::ActiveModel>, DbErr> {
    let model = votes::ActiveModel {
        member: sea_orm::Set(member),
        candidate: sea_orm::Set(candidate),
        ..Default::default()
    };
    
    Votes::insert(model).exec(db).await
}

pub async fn remove(db: &DatabaseConnection, member: i64, candidate: i64) -> Result<sea_orm::DeleteResult, DbErr> {
    Votes::delete_many()
        .filter(
            sea_orm::Condition::all()
            .add(votes::Column::Member.eq(member))
            .add(votes::Column::Candidate.eq(candidate))
        )
        .exec(db)
        .await
}