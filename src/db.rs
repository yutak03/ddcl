use std::process::Stdio;

use tokio::process::Command;

use crate::config::{DatabaseConnection, DatabaseType};
use crate::error::{AppError, Result};

/// データベース接続の抽象
pub struct DatabaseConnector;

impl DatabaseConnector {
    /// データベースに接続する
    pub async fn connect(connection: &DatabaseConnection) -> Result<()> {
        match connection.db_type {
            DatabaseType::PostgreSQL => Self::connect_postgresql(connection).await,
            DatabaseType::MySQL => Self::connect_mysql(connection).await,
            DatabaseType::MongoDB => Self::connect_mongodb(connection).await,
        }
    }

    /// PostgreSQLに接続
    async fn connect_postgresql(connection: &DatabaseConnection) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("exec")
            .arg("-it")
            .arg(&connection.container)
            .arg("psql");

        // データベース名を追加（指定されている場合）
        if let Some(db) = &connection.database {
            cmd.arg("-d").arg(db);
        }

        // ユーザー名を追加
        cmd.arg("-U").arg(&connection.user);

        // 追加オプションがある場合は追加
        if let Some(options) = &connection.options {
            for (key, value) in options {
                cmd.arg(format!("--{}", key)).arg(value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "PostgreSQLコンテナへの接続に失敗しました: {:?}",
                status
            )));
        }

        Ok(())
    }

    /// MySQLに接続
    async fn connect_mysql(connection: &DatabaseConnection) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("exec")
            .arg("-it")
            .arg(&connection.container)
            .arg("mysql");

        // データベース名を追加（指定されている場合）
        if let Some(db) = &connection.database {
            cmd.arg(db);
        }

        // ユーザー名を追加
        cmd.arg("-u").arg(&connection.user);

        // パスワードを追加（指定されている場合）
        if let Some(password) = &connection.password {
            cmd.arg(format!("-p{}", password));
        }

        // 追加オプションがある場合は追加
        if let Some(options) = &connection.options {
            for (key, value) in options {
                cmd.arg(format!("--{}", key)).arg(value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "MySQLコンテナへの接続に失敗しました: {:?}",
                status
            )));
        }

        Ok(())
    }

    /// MongoDBに接続
    async fn connect_mongodb(connection: &DatabaseConnection) -> Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("exec")
            .arg("-it")
            .arg(&connection.container)
            .arg("mongosh");

        // 認証情報を追加（指定されている場合）
        if !connection.user.is_empty() {
            let _auth_string = if let Some(password) = &connection.password {
                format!("{}:{}", connection.user, password)
            } else {
                connection.user.clone()
            };
            cmd.arg("-u").arg(&connection.user);

            if let Some(password) = &connection.password {
                cmd.arg("-p").arg(password);
            }
        }

        // データベース名を追加（指定されている場合）
        if let Some(db) = &connection.database {
            cmd.arg(db);
        }

        // 追加オプションがある場合は追加
        if let Some(options) = &connection.options {
            for (key, value) in options {
                cmd.arg(format!("--{}", key)).arg(value);
            }
        }

        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await?;

        if !status.success() {
            return Err(AppError::Docker(format!(
                "MongoDBコンテナへの接続に失敗しました: {:?}",
                status
            )));
        }

        Ok(())
    }

    /// コンテナが実行中かチェック
    pub async fn check_container(container_name: &str) -> Result<bool> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("--format")
            .arg("{{.Names}}")
            .output()
            .await?;

        if !output.status.success() {
            return Err(AppError::Docker(
                "Dockerコンテナリストの取得に失敗しました".to_string(),
            ));
        }

        let containers = String::from_utf8_lossy(&output.stdout);
        Ok(containers.lines().any(|name| name.trim() == container_name))
    }
}
