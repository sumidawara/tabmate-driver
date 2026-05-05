# tabmate-driver

CLIP STUDIO TABMATE の Linux 用ドライバーです。
Bluetooth で接続し、ボタン入力をキーボード・マウスイベントとして送出します。

## 必要なもの

- Linux
- Rust ツールチェーン（[rustup](https://rustup.rs) 推奨）
- Bluetooth アダプター
- CLIP STUDIO TABMATE

## セットアップ

### 1. クローン

```sh
git clone https://github.com/sumidawara/tabmate-driver.git
cd tabmate-driver
```

### 2. MAC アドレスの設定

`src/driver.rs` の以下の行を自分の TABMATE の MAC アドレスに変更します：

```rust
let tabmate_addr: bluer::Address = "20:73:10:00:D5:52".parse()?;
```

MAC アドレスは `bluetoothctl` で確認できます：

```sh
bluetoothctl
[bluetooth]# scan on   # TABMATE の電源を入れて待つ
[bluetooth]# scan off
```

### 3. 実行

```sh
cargo sudo-run
```

ビルドして root で起動します（uinput / evdev のアクセスに root 権限が必要）。

## キーマップのカスタマイズ

実行ディレクトリに `config.toml` を置くと、デフォルトのキーマップを上書きできます。
設定例は同梱の `config.toml` を参照してください。

## トラブルシューティング

Bluetooth が認識されない場合：

```sh
systemctl status bluetooth
sudo systemctl restart bluetooth
```
