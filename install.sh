#!/usr/bin/env sh
# llmLab One-Command Installer & Updater for Linux & Android Termux
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/tatarhost/llmLab/main/install.sh | sh

set -eu

REPO="${LLMLAB_REPO:-tatarhost/llmLab}"
TMP_DIR="$(mktemp -d 2>/dev/null || echo "${TMPDIR:-/tmp}/llmlab-install-$$")"
mkdir -p "$TMP_DIR"

cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

IS_TERMUX=0
if [ -n "${TERMUX_VERSION:-}" ] || printf '%s' "${PREFIX:-}" | grep -q "com.termux"; then
  IS_TERMUX=1
fi

ARCH="$(uname -m)"
ASSET_NAME=""

if [ "$IS_TERMUX" = "1" ]; then
  INSTALL_BIN_DIR="${PREFIX}/bin"
  DATA_DIR="${HOME}/.llmlab"
  case "$ARCH" in
    aarch64|arm64) ASSET_NAME="llmLab-termux-arm64.tar.gz" ;;
    *) ASSET_NAME="" ;;
  esac
  echo "==> Установка llmLab для Android Termux ($ARCH)..."
else
  INSTALL_BIN_DIR="${HOME}/.local/bin"
  DATA_DIR="${HOME}/.local/share/llmlab"
  case "$ARCH" in
    x86_64|amd64) ASSET_NAME="llmLab-linux-x86_64.tar.gz" ;;
    aarch64|arm64) ASSET_NAME="llmLab-linux-arm64.tar.gz" ;;
    *) ASSET_NAME="" ;;
  esac
  echo "==> Установка llmLab для Linux ($ARCH)..."
fi

mkdir -p "$INSTALL_BIN_DIR" "$DATA_DIR" "$DATA_DIR/models" "$DATA_DIR/workspace"

# Stop running llmLab process and clean old version before upgrading
if command -v pkill >/dev/null 2>&1; then
  pkill -x llmLab 2>/dev/null || true
fi
rm -f "${INSTALL_BIN_DIR}/llmLab" "${INSTALL_BIN_DIR}/llmLab.old" "${DATA_DIR}/models/"*.part

DOWNLOADED=0
if [ -n "$ASSET_NAME" ]; then
  ZIP_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
  ALT_URL="https://github.com/${REPO}/releases/download/latest/${ASSET_NAME}"
  OUT_TAR="${TMP_DIR}/${ASSET_NAME}"

  echo "    Скачивание ${ASSET_NAME} (старая версия удалена)..."
  if curl -fLsS "$ZIP_URL" -o "$OUT_TAR" 2>/dev/null || curl -fLsS "$ALT_URL" -o "$OUT_TAR" 2>/dev/null; then
    tar -xzf "$OUT_TAR" -C "$TMP_DIR"
    if [ -f "${TMP_DIR}/llmLab" ]; then
      cp -f "${TMP_DIR}/llmLab" "${INSTALL_BIN_DIR}/llmLab"
      chmod +x "${INSTALL_BIN_DIR}/llmLab"
      [ -f "${TMP_DIR}/config.example.toml" ] && [ ! -f "${DATA_DIR}/config.toml" ] && cp "${TMP_DIR}/config.example.toml" "${DATA_DIR}/config.toml"
      DOWNLOADED=1
    fi
  fi
fi

if [ "$DOWNLOADED" = "0" ]; then
  echo "    Готовый архив для ${ARCH} ещё не опубликован — выполняю сборку из исходников..."
  if [ "$IS_TERMUX" = "1" ]; then
    pkg update -y >/dev/null 2>&1 || true
    pkg install -y rust git clang >/dev/null 2>&1 || true
  fi

  if ! command -v cargo >/dev/null 2>&1 || ! command -v git >/dev/null 2>&1; then
    echo "Ошибка: требуются git и cargo (Rust) для локальной сборки."
    if [ "$IS_TERMUX" = "1" ]; then
      echo "Выполните: pkg install rust git clang && curl -fsSL https://raw.githubusercontent.com/${REPO}/main/install.sh | sh"
    else
      echo "Установите Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
    exit 1
  fi

  SRC_DIR="${TMP_DIR}/src"
  git clone --depth 1 "https://github.com/${REPO}.git" "$SRC_DIR"
  (cd "$SRC_DIR" && cargo build --release)
  cp -f "${SRC_DIR}/target/release/llmLab" "${INSTALL_BIN_DIR}/llmLab"
  chmod +x "${INSTALL_BIN_DIR}/llmLab"
  if [ -f "${SRC_DIR}/config.example.toml" ] && [ ! -f "${DATA_DIR}/config.toml" ]; then
    cp "${SRC_DIR}/config.example.toml" "${DATA_DIR}/config.toml"
  fi
fi

# Ensure ~/.local/bin is in PATH on Linux
if [ "$IS_TERMUX" = "0" ]; then
  case ":${PATH}:" in
    *":${INSTALL_BIN_DIR}:"*) ;;
    *)
      export PATH="${INSTALL_BIN_DIR}:${PATH}"
      if [ -f "${HOME}/.bashrc" ] && ! grep -q "${INSTALL_BIN_DIR}" "${HOME}/.bashrc" 2>/dev/null; then
        printf '\nexport PATH="%s:$PATH"\n' "$INSTALL_BIN_DIR" >> "${HOME}/.bashrc"
      fi
      if [ -f "${HOME}/.zshrc" ] && ! grep -q "${INSTALL_BIN_DIR}" "${HOME}/.zshrc" 2>/dev/null; then
        printf '\nexport PATH="%s:$PATH"\n' "$INSTALL_BIN_DIR" >> "${HOME}/.zshrc"
      fi
      echo "    Добавлено в PATH: ${INSTALL_BIN_DIR}"
      ;;
  esac
fi

echo "==> Готово! Установлено в ${INSTALL_BIN_DIR}/llmLab"
if [ "$IS_TERMUX" = "1" ]; then
  echo "    Совет для Termux: для локального GGUF движка выполните 'pkg install llama-cpp' (или используйте Cloud Boost)."
fi

if [ -t 0 ] || [ -e /dev/tty ]; then
  exec "${INSTALL_BIN_DIR}/llmLab" </dev/tty
else
  echo "    Запустите командой: llmLab"
fi
