# Docker データベースコンテナ接続ツール

Docker で実行されているデータベースコンテナに簡単に接続するための CLI ツールです。

# 前提条件
ローカルマシンで起動されているDockerを対象にしています。

## 機能

- 複数のデータベースタイプ (PostgreSQL, MySQL, MongoDB) に対応
- エイリアスで簡単に接続できる機能
- コマンドライン引数での直接接続
- 設定ファイルに接続情報を保存

## インストール

### ソースからビルド

```bash
cargo install --path .
```

## 使い方

### 接続設定を追加

```bash
# PostgreSQL コンテナの設定を追加
dbcli add [alias_name] --container [docker_container_name] --db-type postgres --user postgres --password secret --database [database_name]

# MySQL コンテナの設定を追加
dbcli add [alias_name] --container [docker_container_name] --db-type mysql --user root --password secret --database [database_name]

# MongoDB コンテナの設定を追加
dbcli add [alias_name] --container [docker_container_name] --db-type mongodb --user mongo --password secret --database [database_name]
```

### 設定一覧を表示

```bash
dbcli list
```

出力例:
```
接続設定一覧:
  [alias_name]: PostgreSQL (postgres@postgres_container, DB: mydb)
  [alias_name]: MySQL (root@mysql_container, DB: mydb)
  [alias_name]: MongoDB (mongo@mongo_container, DB: admin)
```

### エイリアスを使って接続

```bash
# PostgreSQLコンテナに接続
dbcli connect postgres-dev

# MySQLコンテナに接続
dbcli connect mysql-dev

# MongoDBコンテナに接続
dbcli connect mongo-dev
```

### 直接パラメータを指定して接続

```bash
# PostgreSQLコンテナに直接接続
dbcli connect --container postgres_container --db-type postgres --user postgres --password secret --database mydb

# MySQLコンテナに直接接続
dbcli connect --container mysql_container --db-type mysql --user root --password secret --database mydb

# MongoDBコンテナに直接接続
dbcli connect --container mongo_container --db-type mongodb --user mongo --password secret --database admin
```

### 設定の削除

```bash
dbcli remove postgres-dev
```

## 設定ファイル

設定ファイルは YAML 形式で以下の場所に保存されます:

- Linux: `~/.config/docker_db_container_login/config.yaml`
- macOS: `~/Library/Application Support/docker_db_container_login/config.yaml`
- Windows: `%APPDATA%\docker_db_container_login\config.yaml`

設定ファイルの例:

```yaml
version: "0.0.1"
connections:
  postgres-dev:
    db_type: PostgreSQL
    container: postgres_container
    user: postgres
    password: secret
    database: mydb
  mysql-dev:
    db_type: MySQL
    container: mysql_container
    user: root
    password: secret
    database: mydb
```

## 前提条件

- Dockerがインストールされていること
- 接続するデータベースコンテナが実行中であること
- コンテナ内にデータベースクライアントがインストールされていること:
  - PostgreSQL: `psql`
  - MySQL: `mysql`
  - MongoDB: `mongosh`

## ライセンス

MIT

## 参考

以下のテンプレートから作成しました。

- [rust-cli-template - skanehira](https://github.com/skanehira/rust-cli-template)
