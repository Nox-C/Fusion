use std::env;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DrizzleConfig {
    pub out: String,
    pub schema: String,
    pub dialect: String,
    pub db_credentials: DBCredentials,
}

#[derive(Debug, Serialize)]
pub struct DBCredentials {
    pub url: String,
}

impl DrizzleConfig {
    /// Load configuration from environment, panic if DATABASE_URL is not set.
    pub fn from_env() -> Self {
        let url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set; ensure the database is provisioned");
        DrizzleConfig {
            out: String::from("./migrations"),
            schema: String::from("./shared/schema.ts"),
            dialect: String::from("postgresql"),
            db_credentials: DBCredentials { url },
        }
    }
}
