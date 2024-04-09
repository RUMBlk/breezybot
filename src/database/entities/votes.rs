//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "votes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub member: i64,
    pub candidate: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::candidates::Entity",
        from = "Column::Candidate",
        to = "super::candidates::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Candidates,
    #[sea_orm(
        belongs_to = "super::members::Entity",
        from = "Column::Member",
        to = "super::members::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Members,
}

impl Related<super::candidates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Candidates.def()
    }
}

impl Related<super::members::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Members.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}