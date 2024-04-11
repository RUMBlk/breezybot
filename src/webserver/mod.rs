use poem::{get, handler, post, web::{ Form, Json, Path, Data}, Route, middleware::AddData, EndpointExt};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use serde::Deserialize;
use std::{ops::Deref, sync::Arc};
use poem::middleware::AddDataEndpoint;

use crate::database as db;


#[handler]
fn hello_world() -> &'static str {
    "Hello, world!"
}

#[handler]
async fn database(db: Data<&Arc<DatabaseConnection>>) -> Json<Vec<db::entities::test::Model>> {
    let db: &DatabaseConnection = db.deref().as_ref();
    let tests = db::entities::prelude::Test::find().all(db).await;
    Json(tests.unwrap_or_default())
}

#[derive(Deserialize)]
struct MembersPost {
    authorization: String,
    guild_id: String,
    points: i64,
}

#[handler]
async fn update_members(Path(id): Path<String>, Form(request): Form<MembersPost>, db: Data<&Arc<DatabaseConnection>>, authorization: Data<&Arc<String>>) -> String {
    let db: &DatabaseConnection = db.deref().as_ref();
    match authorization.to_string() == request.authorization {
        true => {
            let mut member: db::entities::members::ActiveModel = db::queries::members::inselect(db, &request.guild_id, &id).await.unwrap().into_active_model();
            member.points = Set(member.points.unwrap() + request.points);
            member.update(db).await.expect("well, oof");
            "Ok!".to_string()
        },
        false => "Auth sucks".to_string(),
    }
}
pub fn poem(db: DatabaseConnection, authorization: String) -> AddDataEndpoint<AddDataEndpoint<poem::Route, Arc<sea_orm::DatabaseConnection>>, Arc<String>> {
    let app = Route::new()
        .at("/hello_world", get(hello_world))
        .at("/api/database", get(database))
        .at("/api/guilds/members/:id", post(update_members))
        .with(AddData::new(Arc::new(db)))
        .with(AddData::new(Arc::new(authorization)));
    app
}