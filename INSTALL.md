# TABMATE Driver — インストール手順

すべて `make` 経由で完結します。`make help` でコマンド一覧が見られます。

## インストール

```sh
sudo make install   # バイナリ・設定・systemd unit を配置
sudo make enable    # サービスを有効化して起動
```

これだけで完了。再起動後も自動で常駐します。

## 動作確認

```sh
make status         # サービスの状態
make logs           # リアルタイムでログを追う (Ctrl+Cで終了)
```

## 設定変更

`config.toml` を編集したら：

```sh
sudo cp config.toml /etc/tabmate-driver/config.toml
sudo make restart
```

## アンインストール

```sh
sudo make uninstall   # サービス停止 + 全ファイル削除
```

## 開発時の運用

開発中は何度もビルド＆配置を繰り返すので、symlinkで配置しておくと楽です：

```sh
sudo make dev-install   # バイナリ・設定・unit を symlink で配置
```

これで `cargo build --release` するだけで `/usr/local/bin/tabmate-driver` も更新されます。`config.toml` の編集も即反映。

開発ループ：

```sh
sudo make dev   # ビルド → 再起動 → ログ追跡
```

## トラブルシューティング

```sh
# 起動失敗の原因を見る
make status
journalctl -u tabmate-driver -n 50

# systemdの設定を再読込 (.service ファイルを変更した時)
sudo systemctl daemon-reload
```

## カスタマイズ

`PREFIX` でインストール先を変えられます (デフォルト: `/usr/local`):

```sh
sudo make install PREFIX=/usr   # /usr/bin にバイナリを置く場合
```
