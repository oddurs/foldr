#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
    echo "Usage: bash scripts/package.sh TARGET [OUTPUT_DIRECTORY]" >&2
    exit 2
fi

foldr_package_target=$1
case "$foldr_package_target" in
    aarch64-apple-darwin|x86_64-apple-darwin|x86_64-unknown-linux-gnu) ;;
    *) echo "Unsupported package target: $foldr_package_target" >&2; exit 2 ;;
esac

foldr_package_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$foldr_package_root"
foldr_package_output=${2:-dist}
mkdir -p "$foldr_package_output"
foldr_package_output=$(cd "$foldr_package_output" && pwd)

cargo build --release --locked -p foldr-cli --target "$foldr_package_target"
foldr_package_binary="$foldr_package_root/target/$foldr_package_target/release/foldr"
foldr_package_version=$("$foldr_package_binary" --version)
foldr_package_version=${foldr_package_version#foldr }
case "$foldr_package_version" in
    ""|*[!0-9A-Za-z.+_-]*) echo "Invalid CLI version" >&2; exit 1 ;;
esac

foldr_package_name="foldr-$foldr_package_version-$foldr_package_target"
foldr_package_temp=$(mktemp -d)
trap 'rm -rf "$foldr_package_temp"' EXIT
foldr_package_dir="$foldr_package_temp/$foldr_package_name"
mkdir -p "$foldr_package_dir/bin" "$foldr_package_dir/share/man/man1" \
    "$foldr_package_dir/share/bash-completion/completions" \
    "$foldr_package_dir/share/zsh/site-functions" \
    "$foldr_package_dir/share/fish/vendor_completions.d"
install -m 755 "$foldr_package_binary" "$foldr_package_dir/bin/foldr"
cp LICENSE README.md "$foldr_package_dir/"
"$foldr_package_binary" man > "$foldr_package_dir/share/man/man1/foldr.1"
"$foldr_package_binary" completions bash > "$foldr_package_dir/share/bash-completion/completions/foldr"
"$foldr_package_binary" completions zsh > "$foldr_package_dir/share/zsh/site-functions/_foldr"
"$foldr_package_binary" completions fish > "$foldr_package_dir/share/fish/vendor_completions.d/foldr.fish"

foldr_package_archive="$foldr_package_output/$foldr_package_name.tar.gz"
COPYFILE_DISABLE=1 tar -czf "$foldr_package_archive" -C "$foldr_package_temp" "$foldr_package_name"
if command -v sha256sum >/dev/null 2>&1; then
    (cd "$foldr_package_output" && sha256sum "$foldr_package_name.tar.gz" > "$foldr_package_name.tar.gz.sha256")
else
    (cd "$foldr_package_output" && shasum -a 256 "$foldr_package_name.tar.gz" > "$foldr_package_name.tar.gz.sha256")
fi
echo "$foldr_package_archive"
