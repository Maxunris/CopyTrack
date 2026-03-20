#!/usr/bin/env bash

set -euo pipefail

TARGET_PATH="${1:-}"

if [[ -z "${TARGET_PATH}" ]]; then
  echo "Usage: scripts/verify-macos-release.sh /path/to/CopyTrack.app|CopyTrack.dmg" >&2
  exit 1
fi

if [[ ! -e "${TARGET_PATH}" ]]; then
  echo "File not found: ${TARGET_PATH}" >&2
  exit 1
fi

if [[ "${TARGET_PATH}" == *.app ]]; then
  echo "Inspecting signed app: ${TARGET_PATH}"
  codesign -dv --verbose=4 "${TARGET_PATH}"
  codesign --verify --deep --strict --verbose=2 "${TARGET_PATH}"
  spctl --assess --type exec --verbose=4 "${TARGET_PATH}"
  xcrun stapler validate "${TARGET_PATH}"
  exit 0
fi

if [[ "${TARGET_PATH}" == *.dmg ]]; then
  echo "Inspecting notarized dmg: ${TARGET_PATH}"
  spctl --assess --type open --verbose=4 "${TARGET_PATH}"
  xcrun stapler validate "${TARGET_PATH}"
  exit 0
fi

echo "Unsupported file type: ${TARGET_PATH}" >&2
exit 1
