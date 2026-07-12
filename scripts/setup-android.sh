#!/usr/bin/env bash
# One-time Android dev environment setup (Ubuntu / WSL).
set -euo pipefail

echo "==> Black Magic IPTV — Android setup"
echo ""

if ! command -v java >/dev/null 2>&1; then
  echo "Installing OpenJDK 17..."
  sudo apt update
  sudo apt install -y openjdk-17-jdk
fi

echo "Java: $(java -version 2>&1 | head -1)"

if [[ -z "${ANDROID_HOME:-}" && -z "${ANDROID_SDK_ROOT:-}" ]]; then
  DEFAULT_SDK="$HOME/Android/Sdk"
  echo ""
  echo "ANDROID_HOME is not set."
  echo "Install Android Studio or the command-line SDK, then add to ~/.bashrc:"
  echo ""
  echo "  export ANDROID_HOME=\"$DEFAULT_SDK\""
  echo "  export PATH=\"\$PATH:\$ANDROID_HOME/platform-tools:\$ANDROID_HOME/cmdline-tools/latest/bin\""
  echo ""
  echo "Required SDK packages (sdkmanager):"
  echo "  platform-tools"
  echo "  platforms;android-36"
  echo "  build-tools;36.0.0"
  echo "  ndk;27.2.12479018"
else
  echo "ANDROID_HOME=${ANDROID_HOME:-$ANDROID_SDK_ROOT}"
fi

echo ""
echo "Installing Rust Android targets..."
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

echo ""
echo "Initialize the Tauri Android project (from repo root):"
echo "  npm install"
echo "  npm run tauri android init -- --ci"
echo ""
echo "Run on emulator/device:"
echo "  npm run tauri:android:dev"
echo ""
echo "Release APK/AAB:"
echo "  npm run tauri:android:build"
echo ""
echo "IMPORTANT: Building Android from WSL requires a LINUX Android SDK/NDK"
echo "  (~/Android/Sdk), NOT the Windows SDK under /mnt/c/.../AppData/Local/Android/Sdk"
echo ""
echo "Recommended: build Android from Windows PowerShell (see README)."
echo "  robocopy from WSL, then: npm run tauri:android:dev"
