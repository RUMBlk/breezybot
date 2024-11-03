use sea_orm::{ *, prelude::{ Expr, Decimal } };
use chrono::TimeDelta;
use sea_query::OnConflict;
use crate::database as db;
#[allow(unused_imports)]
use db::entities::{ *, prelude::* };
/*use super::Error;
use super::sqlmacro::*;*/

pub async fn id(
    db: &DatabaseConnection,
    elections: u64,
    user: u64
) -> Result<Option<i64>, DbErr> {
    Candidates::find()
        .filter(
            sea_orm::Condition::all()
                .add(candidates::Column::Elections.eq(elections))
                .add(candidates::Column::User.eq(user))
        )
        .select_only()
        .column(candidates::Column::Id)
        .into_tuple()
        .one(db)
        .await
}

pub async fn get(
    db: &DatabaseConnection,
    elections: u64,
    user: u64
) -> Result<Option<candidates::Model>, DbErr> {
    Candidates::find()
        .filter(
            sea_orm::Condition::all()
                .add(candidates::Column::Elections.eq(elections))
                .add(candidates::Column::User.eq(user))
        )
        .one(db)
        .await
}

pub async fn is_active(
    db: &DatabaseConnection,
    elections: u64,
    user: u64
) -> Result<Option<bool>, DbErr> {
    Candidates::find()
        .filter(
            sea_orm::Condition::all()
                .add(candidates::Column::Elections.eq(elections))
                .add(candidates::Column::User.eq(user))
        )
        .select_only()
        .column(candidates::Column::Active)
        .into_tuple::<bool>()
        .one(db)
        .await
}

pub async fn register(
    db: &DatabaseConnection,
    elections: u64,
    user: u64
) -> Result<InsertResult<candidates::ActiveModel>, DbErr> {
    let model = candidates::ActiveModel {
        elections: Set(elections.into()),
        user: Set(user.into()),
        active: Set(true),
        ..Default::default()
    };

    Candidates::insert(model)
        .on_conflict(
            OnConflict::columns(vec![candidates::Column::Elections, candidates::Column::User])
                .update_column(candidates::Column::Active)
                .action_and_where(Expr::col((Candidates, candidates::Column::Active)).eq(false))
                .to_owned()
        )
        .exec(db)
        .await
        .inspect_err(|v| eprintln!("{}", v))
}

pub async fn unregister(
    db: &DatabaseConnection,
    elections: u64,
    user: u64
) -> Result<UpdateResult, DbErr> {
    if let Some(id) = id(db, elections, user).await? {
        if let Err(e) = super::votes::remove_candidate_votes(db, id).await { return Err(e) };
    }

    Candidates::update_many()
        .filter(
            sea_orm::Condition::all()
                .add(candidates::Column::Elections.eq(elections))
                .add(candidates::Column::User.eq(user))
                .add(candidates::Column::Active.eq(true))
        )
        .col_expr(candidates::Column::Active, Expr::value(false))
        .exec(db)
        .await
}

pub async fn extend_ban(
    db: &DatabaseConnection,
    elections: u64,
    user: u64,
    duration: chrono::TimeDelta
) -> Result<UpdateResult, DbErr> {
    Candidates::update_many()
        .filter(candidates::Column::Elections.eq(elections))
        .filter(candidates::Column::User.eq(user))
        .col_expr(
            candidates::Column::BannedUntil,
            Expr::cust_with_values("banned_until + interval ? second", vec![duration.num_seconds()])
        ).exec(db).await
}

pub async fn ban(
    db: &DatabaseConnection,
    elections: u64,
    user: u64,
    duration: TimeDelta
) -> Result<candidates::Model, DbErr> {
    let banned_until = chrono::Local::now()
        .checked_add_signed(duration)
        .unwrap_or(chrono::DateTime::<chrono::Utc>::MAX_UTC.into()).into(); //max value when overflow

    let model = candidates::ActiveModel {
        elections: Set(elections.into()),
        user: Set(user.into()),
        banned_until: Set(Some(banned_until)),
        ..Default::default()
    };
    Candidates::insert(model)
        .on_conflict(
            OnConflict::columns(vec![candidates::Column::Elections, candidates::Column::User])
                .value(candidates::Column::BannedUntil, Expr::col(candidates::Column::BannedUntil).add(duration.num_seconds()))
                .action_and_where(
                    Expr::col(candidates::Column::BannedUntil)
                        .is_null()
                        .or(Expr::current_timestamp().gte(Expr::col(candidates::Column::BannedUntil))))
                .to_owned()
        )
        .exec_with_returning(db)
        .await
}

pub async fn unban(db: &DatabaseConnection, elections: u64, user: u64) -> Result<UpdateResult, DbErr> {
    Candidates::update_many()
        .filter(candidates::Column::Elections.eq(elections))
        .filter(candidates::Column::User.eq(user))
        .filter(candidates::Column::BannedUntil.is_not_null())
        .col_expr(
            candidates::Column::BannedUntil,
            Expr::value("NULL"),
        ).exec(db).await
}

pub async fn list_by_voter(db: &DatabaseConnection, role: u64, user: u64) -> Result<Vec<(i64, Decimal)>, DbErr> {
    Candidates::find()
        .inner_join(Votes)
        .inner_join(Elections)
        .filter(
            sea_orm::Condition::all()
            .add(votes::Column::Member.eq(user))
            .add(elections::Column::Role.eq(role))
            .add(candidates::Column::Active.eq(true))
        )
        .select_only()
        .column(candidates::Column::Id)
        .column(candidates::Column::User)
        .into_tuple()
        .all(db)
        .await 
}



/*pub async fn edit_ban(db: &DatabaseConnection, role_id: String, user_id: String, banned_until: Option<chrono::DateTime<chrono::FixedOffset>>) -> Result<candidates::Model, Error> {
    let election = super::elections::find_by_role(role_id).one(db).await.map_err(|e| { Error::Elections(e) })?.unwrap();
    let claim = inselect!(db, candidates, election: election.id, user: user_id.clone()).map_err(|e| { Error::Candidates(e) })?.unwrap();
    update!(db, claim, banned_until).await.map_err(|e| { Error::Candidates(e) })
}*/