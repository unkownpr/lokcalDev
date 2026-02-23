#!/bin/bash
# macOS Production Build Script
# Kullanım: APPLE_PASSWORD='xxxx-xxxx-xxxx-xxxx' ./build-mac.sh

set -e

echo "LokcalDev macOS Build"
echo "====================="

# Sertifika kontrolü
if ! security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
  echo "HATA: Developer ID Application sertifikası bulunamadı."
  exit 1
fi

# Zorunlu env var kontrolü
if [ -z "$APPLE_PASSWORD" ]; then
  echo "HATA: APPLE_PASSWORD eksik."
  echo "  Kullanım: APPLE_PASSWORD='xxxx-xxxx-xxxx-xxxx' ./build-mac.sh"
  exit 1
fi

# Notarization için gerekli env var'ları otomatik set et
export APPLE_ID="programc4@gmail.com"
export APPLE_TEAM_ID="WGWRFNDZXL"

# Tauri updater signing key (local key dosyasından yükle)
if [ -f "$HOME/.tauri/lokcaldev.key" ]; then
  export TAURI_SIGNING_PRIVATE_KEY=$(cat "$HOME/.tauri/lokcaldev.key")
  # Key şifresi varsa: export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="şifren"
  export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
fi

echo "OK: Sertifika bulundu"
echo "OK: Apple credentials set"
echo ""
echo "Building..."

pnpm tauri build

echo ""
echo "Build tamamlandi!"
echo "Cikti: src-tauri/target/release/bundle/macos/"
echo "DMG:   src-tauri/target/release/bundle/dmg/"
