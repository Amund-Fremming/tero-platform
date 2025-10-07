#[cfg(test)]
mod tests {
    use std::env;

    use dotenv::dotenv;
    use sqlx::{Pool, Postgres};
    use tracing::level_filters::LevelFilter;

    use crate::key_vault::key_vault::KeyVault;

    // Setup logging for tests - call this once per test or in a setup function
    fn setup_logging() {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(LevelFilter::DEBUG)
            .with_test_writer() // This makes logs appear in test output
            .try_init(); // Use try_init to avoid panicking if already initialized
    }

    async fn setup_pool() -> Pool<Postgres> {
        dotenv().ok();
        let connection_string =
            env::var("TERO__DATABASE_URL").expect("Failed to obtain connection string");
        let pool = Pool::<Postgres>::connect(&connection_string)
            .await
            .expect("Failed to connect to database");
        pool
    }

    #[tokio::test]
    async fn max_limit_keys() {
        setup_logging();

        let pool = setup_pool().await;
        let vault = KeyVault::new();
        let num_keys: usize = 100 * 100;

        for _ in 0..num_keys {
            let result = vault.create_key(&pool).await;
            assert!(
                result.is_ok(),
                "Failed to create key: {}",
                result.err().unwrap()
            );
        }

        assert_eq!(vault.size().await, num_keys);
    }
}
