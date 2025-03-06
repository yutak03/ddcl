use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

/// データベースの種類
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQLデータベース
    PostgreSQL,
    /// MySQLデータベース
    MySQL,
    /// MongoDBデータベース
    MongoDB,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::MongoDB => write!(f, "MongoDB"),
        }
    }
}

impl std::str::FromStr for DatabaseType {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "postgresql" | "postgres" | "psql" => Ok(DatabaseType::PostgreSQL),
            "mysql" | "mariadb" => Ok(DatabaseType::MySQL),
            "mongodb" | "mongo" => Ok(DatabaseType::MongoDB),
            _ => Err(AppError::UnknownDatabaseType(s.to_string())),
        }
    }
}

/// データベース接続情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConnection {
    /// データベースの種類
    pub db_type: DatabaseType,
    /// コンテナ名
    pub container: String,
    /// ユーザー名
    pub user: String,
    /// パスワード
    pub password: Option<String>,
    /// データベース名
    pub database: Option<String>,
    /// ポート番号
    pub port: Option<u16>,
    /// 追加オプション
    pub options: Option<HashMap<String, String>>,
}

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// バージョン
    pub version: String,
    /// データベース接続エイリアス
    pub connections: HashMap<String, DatabaseConnection>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            connections: HashMap::new(),
        }
    }
}

impl Config {
    /// 新しい設定オブジェクトを作成
    pub fn new() -> Self {
        Default::default()
    }

    /// 設定ファイルのパスを取得
    pub fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "docker_db_container_login")
            .ok_or_else(|| AppError::Config("設定ディレクトリの取得に失敗しました".to_string()))?;

        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).map_err(|e| {
            AppError::Config(format!("設定ディレクトリの作成に失敗しました: {}", e))
        })?;

        Ok(config_dir.join("config.yaml"))
    }

    /// 設定ファイルから読み込み
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_str = fs::read_to_string(&config_path)?;
        let config = serde_yaml::from_str(&config_str)?;
        Ok(config)
    }

    /// 設定をファイルに保存
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let config_str = serde_yaml::to_string(self)?;
        fs::write(&config_path, config_str)?;
        Ok(())
    }

    /// 接続情報を追加
    pub fn add_connection(&mut self, name: String, connection: DatabaseConnection) -> Result<()> {
        self.connections.insert(name, connection);
        self.save()?;
        Ok(())
    }

    /// 接続情報を削除
    pub fn remove_connection(&mut self, name: &str) -> Result<()> {
        if self.connections.remove(name).is_none() {
            return Err(AppError::AliasNotFound(name.to_string()));
        }
        self.save()?;
        Ok(())
    }

    /// エイリアスから接続情報を取得
    pub fn get_connection(&self, name: &str) -> Result<&DatabaseConnection> {
        self.connections
            .get(name)
            .ok_or_else(|| AppError::AliasNotFound(name.to_string()))
    }

    /// 接続情報の一覧を取得
    pub fn list_connections(&self) -> Vec<(&String, &DatabaseConnection)> {
        self.connections.iter().collect()
    }
}
