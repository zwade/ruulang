#!/usr/bin/env bash

set -e

if [[ "$(arch)" == "arm64" ]]; then
    ARCH="aarch64"
elif [[ "$(arch)" == "x86_64" ]]; then
    ARCH="x86_64"
else
    echo "Unsupported architecture: $(arch)"
    exit 1
fi

if [[ "$(uname)" == "Darwin" ]]; then
    OS="apple-darwin"
elif [[ "$(uname)" == "Linux" ]]; then
    OS="unknown-linux-gnu"
else
    echo "Unsupported OS: $(uname)"
    exit 1
fi

FILE_NAME="ruu.${ARCH}-${OS}"

SCRIPT=$(cat <<EOF
import sys, json
assets = [asset for asset in json.load(sys.stdin)['assets'] if asset['name'] == '${FILE_NAME}']
if len(assets) == 0:
    sys.exit(1);
print(assets[0]['url'])
EOF
)

ASSET_URL=$(curl --request GET \
  --url https://api.github.com/repos/zwade/ruulang/releases/latest \
  -Ss \
  --header 'Accept: application/vnd.github+json' \
  | python -c "$SCRIPT"
)

if [[ $? -ne 0 ]]; then
    echo "Failed to find asset for ${ARCH}-${OS}"
    exit 1
fi

echo "Downloaded a new version of ruu from ${ASSET_URL}"

TMPFILE=$(mktemp)

curl --request GET \
  --url "${ASSET_URL}" \
  --header 'Accept: application/octet-stream' \
  -Ss \
  -L \
  --output "${TMPFILE}"

chmod +x "${TMPFILE}"

echo "Installing to /usr/local/bin/ruu"

sudo mv "${TMPFILE}" /usr/local/bin/ruu