#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use dotenv::dotenv;
    use tracing::level_filters::LevelFilter;

    use crate::common::{app_state::AppState, key_vault::KeyVaultError};

    fn setup_logging() {
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(LevelFilter::DEBUG)
            .with_test_writer()
            .try_init()
            .unwrap();
    }

    async fn setup_app_state() -> Arc<AppState> {
        dotenv().ok();
        let connection_string =
            env::var("TERO__DATABASE_URL").expect("Failed to obtain connection string");
        let state = AppState::from_connection_string(&connection_string)
            .await
            .unwrap();
        state
    }

    #[tokio::test]
    async fn max_limit_keys() {
        setup_logging();
        let state = setup_app_state().await;
        let vault = state.get_vault();

        for num in 0..10000 {
            let word = vault.create_key(state.syslog()).await.unwrap();
            println!("{} - {}", num + 1, word)
        }

        let result = vault.create_key(state.syslog()).await;
        assert!(result.is_err());

        let error = result.err().unwrap();
        match error {
            KeyVaultError::FullCapasity => assert!(true),
            _ => assert!(false, "Failed with: {}", error.to_string()),
        }
    }
}
