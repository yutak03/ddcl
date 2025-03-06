use anyhow::Context;
use clap::Parser;
use docker_db_container_login::{
    Config, DatabaseConnector, Result,
    cli::{Commands, ConnectArgs},
};
use std::process;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ロガーを初期化
    env_logger::init();

    // CLIを解析
    let cli = docker_db_container_login::Cli::parse();

    // 設定をロード
    let mut config = Config::load().context("設定のロードに失敗しました")?;

    // サブコマンドを実行
    match cli.command {
        Commands::Connect(args) => connect_command(args, &config).await?,
        Commands::Add(args) => {
            let alias = args.alias.clone();
            let connection = args
                .to_connection()
                .map_err(|e| anyhow::anyhow!("接続情報の変換に失敗しました: {}", e))?;

            config
                .add_connection(alias.clone(), connection)
                .context("接続設定の追加に失敗しました")?;

            println!("接続設定 '{}' を追加しました", alias);
        }
        Commands::Remove(args) => {
            config
                .remove_connection(&args.alias)
                .context("接続設定の削除に失敗しました")?;

            println!("接続設定 '{}' を削除しました", args.alias);
        }
        Commands::List => {
            let connections = config.list_connections();

            if connections.is_empty() {
                println!("保存された接続設定はありません");
                return Ok(());
            }

            println!("接続設定一覧:");
            for (alias, conn) in connections {
                println!(
                    "  {}: {} ({}@{}, DB: {})",
                    alias,
                    conn.db_type,
                    conn.user,
                    conn.container,
                    conn.database.as_deref().unwrap_or("-")
                );
            }
        }
    }

    Ok(())
}

/// 接続コマンドを処理
async fn connect_command(args: ConnectArgs, config: &Config) -> Result<()> {
    // 接続情報を取得
    let connection = if let Some(alias) = args.alias {
        // エイリアスから取得
        config.get_connection(&alias)?.clone()
    } else if let Some(connection) = args.to_connection() {
        // コマンドライン引数から作成
        connection
    } else {
        eprintln!(
            "エラー: エイリアスまたは必要なパラメータ（コンテナ名、DB種類、ユーザー名）を指定してください"
        );
        process::exit(1);
    };

    // コンテナが実行中かチェック
    if !DatabaseConnector::check_container(&connection.container).await? {
        eprintln!(
            "エラー: コンテナ '{}' は実行されていません",
            connection.container
        );
        process::exit(1);
    }

    // データベースに接続
    println!(
        "{}コンテナ '{}' に接続しています...",
        connection.db_type, connection.container
    );
    DatabaseConnector::connect(&connection).await?;

    Ok(())
}
