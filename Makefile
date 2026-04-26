PREFIX     ?= /usr/local
BIN_DIR     = $(PREFIX)/bin
CONFIG_DIR  = /etc/tabmate-driver
SYSTEMD_DIR = /etc/systemd/system
SERVICE     = tabmate-driver.service
BIN         = tabmate-driver

.PHONY: help build install uninstall reinstall dev-install \
        enable disable start stop restart status logs dev clean

help:
	@echo "TABMATE Driver — Make targets"
	@echo ""
	@echo "  make build        - リリースビルド"
	@echo "  sudo make install - インストール (バイナリ + 設定 + systemd unit)"
	@echo "  sudo make uninstall - アンインストール"
	@echo "  sudo make reinstall - 入れ直し (uninstall → install)"
	@echo "  sudo make dev-install - 開発用インストール (symlinkで配置)"
	@echo ""
	@echo "  sudo make enable  - サービス有効化 + 起動"
	@echo "  sudo make disable - サービス停止 + 自動起動無効化"
	@echo "  sudo make restart - 再起動"
	@echo "       make status  - 状態表示"
	@echo "       make logs    - ログをリアルタイム表示"
	@echo ""
	@echo "  sudo make dev     - ビルド → 再起動 → ログ追跡 (開発ループ)"
	@echo "       make clean   - cargo clean"

build:
	cargo build --release

install: build
	install -Dm755 target/release/$(BIN) $(BIN_DIR)/$(BIN)
	install -Dm644 config.toml $(CONFIG_DIR)/config.toml
	install -Dm644 $(SERVICE) $(SYSTEMD_DIR)/$(SERVICE)
	systemctl daemon-reload
	@echo ""
	@echo "✓ インストール完了"
	@echo "  起動するには: sudo make enable"

uninstall:
	-systemctl disable --now tabmate-driver 2>/dev/null || true
	rm -f $(SYSTEMD_DIR)/$(SERVICE)
	rm -f $(BIN_DIR)/$(BIN)
	rm -rf $(CONFIG_DIR)
	systemctl daemon-reload
	@echo "✓ アンインストール完了"

reinstall: uninstall install

# 開発用: バイナリ・設定・unit を symlink で配置。
# ソースを編集してビルドし直すだけで反映される (再 install 不要)。
dev-install: build
	mkdir -p $(CONFIG_DIR)
	ln -sf $(CURDIR)/target/release/$(BIN) $(BIN_DIR)/$(BIN)
	ln -sf $(CURDIR)/config.toml $(CONFIG_DIR)/config.toml
	ln -sf $(CURDIR)/$(SERVICE) $(SYSTEMD_DIR)/$(SERVICE)
	systemctl daemon-reload
	@echo ""
	@echo "✓ 開発用インストール完了 (symlink)"
	@echo "  ビルド後に sudo make restart で反映できます"

enable:
	systemctl enable --now tabmate-driver

disable:
	systemctl disable --now tabmate-driver

start:
	systemctl start tabmate-driver

stop:
	systemctl stop tabmate-driver

restart:
	systemctl restart tabmate-driver

status:
	systemctl status tabmate-driver

logs:
	journalctl -u tabmate-driver -f

# 開発ループ: ビルド → 再起動 → ログ追跡
dev: build
	systemctl restart tabmate-driver
	journalctl -u tabmate-driver -f

clean:
	cargo clean
