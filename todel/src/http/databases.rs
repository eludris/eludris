use rocket_db_pools::{deadpool_redis::Pool, sqlx::PgPool, Database};

#[derive(Database)]
#[database("db")]
pub struct DB(pub PgPool);

#[derive(Database)]
#[database("cache")]
pub struct Cache(pub Pool);
