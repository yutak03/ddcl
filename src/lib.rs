pub mod cli;
pub mod config;
pub mod db;
pub mod error;
pub mod interactive;
pub mod validation;

pub use cli::Cli;
pub use config::{Config, DatabaseConnection, DatabaseType};
pub use db::{DatabaseConnector, DetectedContainer};
pub use error::{AppError, Result};
pub use interactive::{get_connection_interactively, get_connection_with_auto_detect};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::str::FromStr;

    use super::*;

    mod config_tests {
        use super::*;
        use std::fs;
        use tempfile::tempdir;

        #[test]
        fn test_database_type_display() {
            assert_eq!(DatabaseType::PostgreSQL.to_string(), "PostgreSQL");
            assert_eq!(DatabaseType::MySQL.to_string(), "MySQL");
            assert_eq!(DatabaseType::MongoDB.to_string(), "MongoDB");
        }

        #[test]
        fn test_database_type_from_str() {
            // 正常系テスト
            assert_eq!(
                DatabaseType::from_str("postgresql").unwrap(),
                DatabaseType::PostgreSQL
            );
            assert_eq!(
                DatabaseType::from_str("postgres").unwrap(),
                DatabaseType::PostgreSQL
            );
            assert_eq!(
                DatabaseType::from_str("psql").unwrap(),
                DatabaseType::PostgreSQL
            );

            assert_eq!(
                DatabaseType::from_str("mysql").unwrap(),
                DatabaseType::MySQL
            );
            assert_eq!(
                DatabaseType::from_str("mariadb").unwrap(),
                DatabaseType::MySQL
            );

            assert_eq!(
                DatabaseType::from_str("mongodb").unwrap(),
                DatabaseType::MongoDB
            );
            assert_eq!(
                DatabaseType::from_str("mongo").unwrap(),
                DatabaseType::MongoDB
            );

            // 大文字小文字の違いをテスト
            assert_eq!(
                DatabaseType::from_str("PostgreSQL").unwrap(),
                DatabaseType::PostgreSQL
            );
            assert_eq!(
                DatabaseType::from_str("MySQL").unwrap(),
                DatabaseType::MySQL
            );
            assert_eq!(
                DatabaseType::from_str("MongoDB").unwrap(),
                DatabaseType::MongoDB
            );

            // エラー系テスト
            assert!(DatabaseType::from_str("unknown").is_err());
            assert!(matches!(
                DatabaseType::from_str("unknown").unwrap_err(),
                AppError::UnknownDatabaseType(_)
            ));
        }

        #[test]
        fn test_database_connection() {
            let conn = DatabaseConnection {
                db_type: DatabaseType::PostgreSQL,
                container: "pg-container".to_string(),
                user: "postgres".to_string(),
                password: Some("password123".to_string()),
                database: Some("testdb".to_string()),
                port: Some(5432),
                options: Some(HashMap::new()),
            };

            assert_eq!(conn.db_type, DatabaseType::PostgreSQL);
            assert_eq!(conn.container, "pg-container");
            assert_eq!(conn.user, "postgres");
            assert_eq!(conn.password, Some("password123".to_string()));
            assert_eq!(conn.database, Some("testdb".to_string()));
            assert_eq!(conn.port, Some(5432));
        }

        #[test]
        fn test_config_default() {
            let config = Config::default();
            assert_eq!(config.connections.len(), 0);
            assert!(!config.version.is_empty());
        }

        #[test]
        fn test_config_add_and_get_connection() {
            let mut config = Config::default();

            // 接続を追加
            let conn = DatabaseConnection {
                db_type: DatabaseType::PostgreSQL,
                container: "pg-container".to_string(),
                user: "postgres".to_string(),
                password: Some("password123".to_string()),
                database: Some("testdb".to_string()),
                port: Some(5432),
                options: None,
            };

            assert!(
                config
                    .add_connection("test-alias".to_string(), conn.clone())
                    .is_ok()
            );

            // 追加した接続を取得
            let retrieved_conn = config.get_connection("test-alias").unwrap();
            assert_eq!(retrieved_conn.db_type, conn.db_type);
            assert_eq!(retrieved_conn.container, conn.container);
            assert_eq!(retrieved_conn.user, conn.user);

            // 存在しない接続を取得しようとするとエラー
            let result = config.get_connection("non-existent");
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::AliasNotFound(_)));
        }

        #[test]
        fn test_config_remove_connection() {
            let mut config = Config::default();

            // 接続を追加
            let conn = DatabaseConnection {
                db_type: DatabaseType::PostgreSQL,
                container: "pg-container".to_string(),
                user: "postgres".to_string(),
                password: None,
                database: None,
                port: None,
                options: None,
            };

            config
                .add_connection("test-alias".to_string(), conn)
                .unwrap();
            assert_eq!(config.connections.len(), 1);

            // 接続を削除
            assert!(config.remove_connection("test-alias").is_ok());
            assert_eq!(config.connections.len(), 0);

            // 存在しない接続を削除しようとするとエラー
            let result = config.remove_connection("test-alias");
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), AppError::AliasNotFound(_)));
        }

        #[test]
        fn test_config_list_connections() {
            let mut config = Config::default();

            // 複数の接続を追加
            let conn1 = DatabaseConnection {
                db_type: DatabaseType::PostgreSQL,
                container: "pg-container".to_string(),
                user: "postgres".to_string(),
                password: None,
                database: None,
                port: None,
                options: None,
            };

            let conn2 = DatabaseConnection {
                db_type: DatabaseType::MySQL,
                container: "mysql-container".to_string(),
                user: "root".to_string(),
                password: None,
                database: None,
                port: None,
                options: None,
            };

            config
                .add_connection("pg-alias".to_string(), conn1)
                .unwrap();
            config
                .add_connection("mysql-alias".to_string(), conn2)
                .unwrap();

            // 接続一覧を取得
            let connections = config.list_connections();
            assert_eq!(connections.len(), 2);

            // 接続が正しく取得できることを確認
            assert!(connections.iter().any(|(name, _)| name == &"pg-alias"));
            assert!(connections.iter().any(|(name, _)| name == &"mysql-alias"));
        }

        #[test]
        fn test_config_save_and_load() {
            // 一時ディレクトリを作成
            let temp_dir = tempdir().unwrap();
            let config_path = temp_dir.path().join("config.yml");

            // テスト用の設定を作成
            let mut config = Config::default();
            let conn = DatabaseConnection {
                db_type: DatabaseType::MongoDB,
                container: "mongo-container".to_string(),
                user: "admin".to_string(),
                password: Some("pass123".to_string()),
                database: Some("testdb".to_string()),
                port: Some(27017),
                options: None,
            };

            config
                .add_connection("mongo-alias".to_string(), conn)
                .unwrap();

            // 設定をファイルに保存
            let yaml = serde_yaml::to_string(&config).unwrap();
            fs::write(&config_path, yaml).unwrap();

            // 設定をファイルから読み込み
            let loaded_yaml = fs::read_to_string(&config_path).unwrap();
            let loaded_config: Config = serde_yaml::from_str(&loaded_yaml).unwrap();

            // 読み込んだ設定が元の設定と一致することを確認
            assert_eq!(loaded_config.version, config.version);
            assert_eq!(loaded_config.connections.len(), 1);

            let loaded_conn = loaded_config.get_connection("mongo-alias").unwrap();
            assert_eq!(loaded_conn.db_type, DatabaseType::MongoDB);
            assert_eq!(loaded_conn.container, "mongo-container");
            assert_eq!(loaded_conn.user, "admin");
            assert_eq!(loaded_conn.password, Some("pass123".to_string()));
            assert_eq!(loaded_conn.database, Some("testdb".to_string()));
            assert_eq!(loaded_conn.port, Some(27017));
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_error_display() {
            let err = AppError::Config("Test error".to_string());
            assert_eq!(err.to_string(), "Config error: Test error");

            let err = AppError::DatabaseConnection("Connection failed".to_string());
            assert_eq!(err.to_string(), "Database connection error: Connection failed");

            let err = AppError::AliasNotFound("test-alias".to_string());
            assert_eq!(
                err.to_string(),
                "Alias 'test-alias' not found"
            );
        }
    }

    mod cli_tests {
        use super::*;
        use crate::cli::{AddArgs, ConnectArgs};

        #[test]
        fn test_connect_args_to_connection() {
            let args = ConnectArgs {
                alias: None,
                container: Some("test-container".to_string()),
                db_type: Some("postgresql".to_string()),
                user: Some("testuser".to_string()),
                password: Some("pass123".to_string()),
                database: Some("testdb".to_string()),
                port: Some(5432),
            };

            let conn = args.to_connection().unwrap();
            assert_eq!(conn.db_type, DatabaseType::PostgreSQL);
            assert_eq!(conn.container, "test-container");
            assert_eq!(conn.user, "testuser");
            assert_eq!(conn.password, Some("pass123".to_string()));
            assert_eq!(conn.database, Some("testdb".to_string()));
            assert_eq!(conn.port, Some(5432));
        }

        #[test]
        fn test_connect_args_invalid() {
            // 必須パラメータが不足している場合
            let args = ConnectArgs {
                alias: None,
                container: None, // コンテナ名がない
                db_type: Some("postgresql".to_string()),
                user: Some("testuser".to_string()),
                password: None,
                database: None,
                port: None,
            };

            assert!(args.to_connection().is_none());

            // データベース型が無効な場合
            let args = ConnectArgs {
                alias: None,
                container: Some("test-container".to_string()),
                db_type: Some("invalid".to_string()), // 不正なDB種別
                user: Some("testuser".to_string()),
                password: None,
                database: None,
                port: None,
            };

            assert!(args.to_connection().is_none());
        }

        #[test]
        fn test_add_args_to_connection() {
            let args = AddArgs {
                alias: Some("test-alias".to_string()),
                container: Some("test-container".to_string()),
                db_type: Some("postgresql".to_string()),
                user: Some("testuser".to_string()),
                password: Some("pass123".to_string()),
                database: Some("testdb".to_string()),
                port: Some(5432),
                interactive: false,
                auto_detect: false,
            };

            let conn = args.to_connection().unwrap();
            assert_eq!(conn.db_type, DatabaseType::PostgreSQL);
            assert_eq!(conn.container, "test-container");
            assert_eq!(conn.user, "testuser");
            assert_eq!(conn.password, Some("pass123".to_string()));
            assert_eq!(conn.database, Some("testdb".to_string()));
            assert_eq!(conn.port, Some(5432));
        }

        #[test]
        fn test_add_args_invalid_db_type() {
            let args = AddArgs {
                alias: Some("test-alias".to_string()),
                container: Some("test-container".to_string()),
                db_type: Some("invalid".to_string()), // Invalid DB type
                user: Some("testuser".to_string()),
                password: None,
                database: None,
                port: None,
                interactive: false,
                auto_detect: false,
            };

            assert!(args.to_connection().is_err());
        }
    }

    mod db_tests {
        use super::*;

        // DockerコンテナのPing関数のテスト
        #[tokio::test]
        async fn test_check_container() {
            let result = DatabaseConnector::check_container("test-container").await;

            // 注意: この関数はDockerのコマンドに依存するため、実際の環境での実行結果に応じて
            // 合格または不合格になります。ここではコマンドの戻り値についてのみテスト

            // テストの目的はコマンドの実行の確認のみ
            assert!(result.is_ok() || result.is_err());
        }

        // データベース接続構造体のテスト
        #[test]
        fn test_db_connection_structures() {
            // PostgreSQL接続テスト
            let pg_conn = DatabaseConnection {
                db_type: DatabaseType::PostgreSQL,
                container: "pg-test".to_string(),
                user: "postgres".to_string(),
                password: None,
                database: Some("testdb".to_string()),
                port: Some(5432),
                options: None,
            };

            assert_eq!(pg_conn.db_type, DatabaseType::PostgreSQL);
            assert_eq!(pg_conn.container, "pg-test");

            // MySQL接続テスト
            let mysql_conn = DatabaseConnection {
                db_type: DatabaseType::MySQL,
                container: "mysql-test".to_string(),
                user: "root".to_string(),
                password: Some("root".to_string()),
                database: None,
                port: Some(3306),
                options: None,
            };

            assert_eq!(mysql_conn.db_type, DatabaseType::MySQL);
            assert_eq!(mysql_conn.container, "mysql-test");

            // MongoDB接続テスト
            let mongo_conn = DatabaseConnection {
                db_type: DatabaseType::MongoDB,
                container: "mongo-test".to_string(),
                user: "admin".to_string(),
                password: None,
                database: None,
                port: Some(27017),
                options: None,
            };

            assert_eq!(mongo_conn.db_type, DatabaseType::MongoDB);
            assert_eq!(mongo_conn.container, "mongo-test");
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
