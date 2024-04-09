use sea_orm::{ EntityTrait, QueryFilter, ColumnTrait, QueryOrder, QuerySelect };

use super::super::entities;
use entities::*;

pub fn sum_votes_for_election(election_id: i64) -> sea_orm::Select<votes::Entity> {
    votes::Entity::find()
    .inner_join(members::Entity)
    .inner_join(candidates::Entity)
    .filter(candidates::Column::Election.eq(election_id))
    .group_by(candidates::Column::Id)
    .order_by_desc(members::Column::Points.sum())
}