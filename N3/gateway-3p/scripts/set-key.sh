#!/bin/sh
set -eu

provider=${1:-}
case "$provider" in
  rapid|qwen) key_name=GATEWAY_RAPID_MLX_KEY ;;
  reflection|beam) key_name=GATEWAY_REFLECTION_KEY ;;
  meta|muse) key_name=GATEWAY_META_KEY ;;
  nvidia|nemotron) key_name=GATEWAY_NVIDIA_KEY ;;
  *)
    printf 'usage: %s {qwen|beam|muse|nemotron}\n' "$0" >&2
    exit 2
    ;;
esac

config_dir=${XDG_CONFIG_HOME:-"$HOME/.config"}/gateway-3p
config_file=$config_dir/keys.env
umask 077
mkdir -p "$config_dir"

printf 'API key for %s: ' "$provider" >&2
if [ -t 0 ]; then stty -echo; fi
IFS= read -r secret
if [ -t 0 ]; then stty echo; fi
printf '\n' >&2
if [ -z "$secret" ]; then
  printf 'refusing to store an empty key\n' >&2
  exit 2
fi

tmp=$(mktemp "$config_file.tmp.XXXXXX")
trap 'rm -f "$tmp"' EXIT HUP INT TERM
if [ -f "$config_file" ]; then
  awk -F= -v wanted="$key_name" '$1 != wanted { print }' "$config_file" > "$tmp"
fi
printf '%s=%s\n' "$key_name" "$secret" >> "$tmp"
chmod 600 "$tmp"
mv "$tmp" "$config_file"
trap - EXIT HUP INT TERM
unset secret

launchctl kickstart -k "gui/$(id -u)/com.stewartbarteau.gateway-3p"
printf '%s stored; gateway restarted\n' "$key_name"
