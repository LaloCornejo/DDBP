use sqlx::PgPool;
use sqlx::migrate::MigrateError;

pub async fn run_migrations(pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!().run(pool).await
}
