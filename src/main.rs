mod database;
mod webserver;
mod discord;

use sea_orm::DatabaseConnection;
use shuttle_secrets::SecretStore;
use std::sync::Arc;
use std::path::Path;
use std::fs::File;
use poem::middleware::AddDataEndpoint;

struct Data {
    mode: String,
    token: String,
    debug_guild: Option<u64>,
    db: Option<sea_orm::DatabaseConnection>,
    //translations: discord::translation::Translations,
}

impl Data {
    pub fn new(mode: String, token: String) -> Self {
        //let translations = discord::translation::read_ftl().expect("failed to read translation files");
        Self { mode, token, debug_guild: None, db: None, /*translations*/ }
    }

    pub fn debug_guild(mut self, debug_guild: u64) -> Self {
        self.debug_guild = Some(debug_guild);
        self
    } 

    pub fn db_option(mut self, db: Option<DatabaseConnection>) -> Self {
        self.db = db;
        self
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[macro_use]
extern crate rust_i18n;
i18n!("locales");

struct WebServer {
    port: String,
    route: Option<AddDataEndpoint<AddDataEndpoint<poem::Route, Arc<sea_orm::DatabaseConnection>>, Arc<String>>>,
}
struct Services {
    webserver: WebServer,
    discord_bot:
        Option<poise::FrameworkBuilder<Data, Box<(dyn std::error::Error +
         std::marker::Send + Sync + 'static)>>>,
}

impl Services {
    async fn webserver(webserver: WebServer) -> Option<()> {
        match webserver.route {
            Some(route) => {
                let server = poem::Server::new(poem::listener::TcpListener::bind(format!("0.0.0.0:{port}", port=webserver.port)));
                Some(server.run(route).await.expect("oof"))
            }
            None => None,
        }
    }
    
    async fn discord_bot(framework: Option<poise::FrameworkBuilder<Data, Box<(dyn std::error::Error +
        std::marker::Send + Sync + 'static)>>>) -> Option<()> {
        match framework {
            Some(framework) => Some(framework.run().await.expect("oof")),
            None => None,
        }
    }
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for Services {
    async fn bind(self, _addr: std::net::SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let webserver = self.webserver;
        let discord_bot = self.discord_bot;

        tokio::select!(
            _ = Services::webserver(webserver) => {},
            _ = Services::discord_bot(discord_bot) => {},
        );

        Ok(())
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> Result<Services, shuttle_runtime::Error> {
    rust_i18n::set_locale("en_US");
    let mode = secret_store.get("MODE").unwrap_or("DEBUG".to_string());

    let db_uri = match "PRODUCTION" {
        "PRODUCTION" => {
            match (
                secret_store.get("DB_HOST"),
                secret_store.get("DB_NAME"),
                secret_store.get("DB_PORT"),
                secret_store.get("DB_USER"),
                secret_store.get("DB_PASS"),
            ) {
                (Some(host), Some(name), Some(port), Some(user), Some(pass)) => Some(database::Postgres::new(host, name, port, user, pass).to_string()),
                _ => None,
            }
        }
        "DEBUG" => { 
            let db_path = "../schema.sqlite"; 
            let mut init = false;

            let file_path = Path::new(&db_path);
            if !file_path.exists() {
                File::create(&file_path).expect("Failed to create file");
                init = true;
            };
            let uri ="sqlite://".to_owned().to_string()+&file_path.canonicalize().expect("fuck!").to_string_lossy().to_string();
            //if init { database::setup_schema(uri.clone()).await; };
            Some(uri)
        },
        _ => None,
    };

    let db = match db_uri.clone() {
        Some(uri) => {
            database::connect(uri).await 
        },
        None => None,
    };

    let webserver = WebServer { 
        port: secret_store.get("WEBSERVER_PORT").unwrap_or("0".to_string()),
        route: match db {
            Some(ref db) => Some(webserver::poem(db.clone(), secret_store.get("AUTHORIZATION").unwrap_or("0000".to_string()))),
            None => None,
        } 
    };

    let discord_bot = match (
        match mode.as_str() {
            "PRODUCTION" => secret_store.get("DISCORD_TOKEN"),
            "DEBUG" => secret_store.get("DISCORD_DEBUG_TOKEN"),
            _ => None,
        },
        match secret_store.get("DEBUG_GUILD") {
            Some(str) => match str.parse::<u64>() {
                Ok(num) => Ok(num),
                _ => Err(()),
            },
            _ => Err(()),
        },
    ) {
        (Some(token), Ok(debug_guild)) => {
            Some(
                discord::build(
                    Data::new(mode, token)
                    .debug_guild(debug_guild)
                    .db_option(db)
                )
                .await
            )
        }
        _ => None
    };

    Ok(Services{webserver, discord_bot})
}