use crate::config::{DatabaseConnection, DatabaseType};
use crate::error::{AppError, Result};
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use std::str::FromStr;

// dialoguer::Errorを変換するためのFromトレイト実装を追加
impl From<dialoguer::Error> for AppError {
    fn from(err: dialoguer::Error) -> Self {
        AppError::Other(err.to_string())
    }
}

/// インタラクティブに接続情報を取得
pub fn get_connection_interactively() -> Result<(String, DatabaseConnection)> {
    let theme = ColorfulTheme::default();

    // エイリアス名の入力
    let alias: String = Input::with_theme(&theme)
        .with_prompt("alias name")
        .interact()?;

    // コンテナ名の入力
    let container: String = Input::with_theme(&theme)
        .with_prompt("Docker container name")
        .interact()?;

    // データベースタイプの選択
    let db_types = &["PostgreSQL", "MySQL", "MongoDB"];
    let db_type_index = Select::with_theme(&theme)
        .with_prompt("Database type")
        .items(db_types)
        .default(0)
        .interact()?;
    let db_type = DatabaseType::from_str(db_types[db_type_index])?;

    // ユーザー名の入力（デフォルト値をデータベースタイプに応じて設定）
    let default_user = match db_type {
        DatabaseType::PostgreSQL => "postgres",
        DatabaseType::MySQL => "root",
        DatabaseType::MongoDB => "mongo",
    };
    let user: String = Input::with_theme(&theme)
        .with_prompt("DB username")
        .default(default_user.to_string())
        .interact()?;

    // パスワードの入力（オプション）
    let password: String = Password::with_theme(&theme)
        .with_prompt("Password (Optional)")
        .allow_empty_password(true)
        .interact()?;
    let password = if password.is_empty() { None } else { Some(password) };

    // データベース名の入力（オプション）
    let database: String = Input::with_theme(&theme)
        .with_prompt("Database name(Optional)")
        .allow_empty(true)
        .interact()?;
    let database = if database.is_empty() { None } else { Some(database) };

    // ポート番号の入力（オプション）
    let port_str: String = Input::with_theme(&theme)
        .with_prompt("Port number (Optional)")
        .allow_empty(true)
        .interact()?;
    let port = if port_str.is_empty() {
        None
    } else {
        match port_str.parse::<u16>() {
            Ok(p) => Some(p),
            Err(_) => {
                println!("Warning: Invalid port number provided, using default port.");
                None
            }
        }
    };

    // 接続情報を作成
    let connection = DatabaseConnection {
        db_type,
        container,
        user,
        password,
        database,
        port,
        options: None,
    };

    Ok((alias, connection))
}