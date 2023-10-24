#!/usr/bin/env bash

author="trvswgnr"
repo="travvy-project-manager"
name="tpm"

arch=$(uname -m)
os=$(uname -s)

case "$os" in
    Darwin*) target="$arch-apple-darwin";;
    Linux*) target="$arch-unknown-linux-gnu";;
    *) echo "unsupported OS"; exit 1;;
esac

binary="$name-$target"

latest_version=$(curl -s "https://api.github.com/repos/${author}/${repo}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
download_url="https://github.com/${author}/${repo}/releases/download/${latest_version}/${binary}.tar.gz"

echo "Downloading ${download_url}..."
curl -sL "${download_url}" | tar xz
chmod +x $binary
mv $binary /usr/local/bin/$name
echo ""
echo "Successfully installed $name $latest_version!"
echo ""
echo "Run '$name --help' to get started"
