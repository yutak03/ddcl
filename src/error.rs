use thiserror::Error;

/// アプリケーションのエラー型
#[derive(Debug, Error)]
pub enum AppError {
    /// 設定ファイルに関連するエラー
    #[error("設定エラー: {0}")]
    Config(String),

    /// データベース接続に関連するエラー
    #[error("データベース接続エラー: {0}")]
    DatabaseConnection(String),

    /// Dockerコンテナに関連するエラー
    #[error("Dockerエラー: {0}")]
    Docker(String),

    /// 不明なデータベースタイプに関連するエラー
    #[error("不明なデータベースタイプ: {0}")]
    UnknownDatabaseType(String),

    /// エイリアスが見つからないエラー
    #[error("エイリアス '{0}' が見つかりませんでした")]
    AliasNotFound(String),

    /// I/O エラー
    #[error("I/O エラー: {0}")]
    Io(#[from] std::io::Error),

    /// YAMLシリアライズ/デシリアライズエラー
    #[error("YAMLエラー: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// その他のエラー
    #[error("エラー: {0}")]
    Other(String),
}

/// 結果型のエイリアス
pub type Result<T> = std::result::Result<T, AppError>;
