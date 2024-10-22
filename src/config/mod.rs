use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
pub mod jwt_middleware;

pub async fn database_connection() -> Result<MySqlPool, sqlx::Error> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = MySqlPool::connect(&database_url).await?;
    println!("Connected to database {}", &database_url);
    Ok(pool)
}

pub mod webauth_utilities;
pub use webauth_utilities::*;
pub use jwt_middleware::*;