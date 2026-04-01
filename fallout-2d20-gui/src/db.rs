// src/db.rs
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use anyhow::Result;
use std::str::FromStr;

pub struct Db {
    pub pool: SqlitePool,
    pub runtime: tokio::runtime::Runtime,
}

impl Db {
    pub fn connect(db_path: &str) -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;

        let options = SqliteConnectOptions::from_str(db_path)?
            .create_if_missing(true)  // creates the .db file on first run
            .pragma("foreign_keys", "ON");

        let pool = runtime.block_on(async {
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await
        })?;

        /*
        // Run migrations on connect
        runtime.block_on(async {
            sqlx::migrate!("./migrations").run(&pool).await
        })?;
        */
        Ok(Self { pool, runtime })
    }

    pub fn block_on<F, T>(&self, fut: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        self.runtime.block_on(fut).map_err(anyhow::Error::from)
    }
}