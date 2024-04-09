use sea_orm::DatabaseConnection;

pub mod entities;
pub mod queries;

pub struct Postgres {
    host: String,
    name: String,
    port: String,
    user: String,
    pass: String,
}

impl Postgres {
    pub fn new(host: String, name: String, port: String, user: String, pass: String) -> Self {
        Self {
            host,
            name,
            port,
            user,
            pass,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.pass, self.host, self.port, self.name
        )
    }
}

pub async fn connect(uri: String) -> Option<DatabaseConnection> {
    let connection = match sea_orm::Database::connect(uri).await {
        Ok(connection) => Some(connection),
        Err(_) => None,
    };
    connection
}