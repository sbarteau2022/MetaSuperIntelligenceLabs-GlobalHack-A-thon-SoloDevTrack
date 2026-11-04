#!/bin/sh
# Site preview. If macOS blocks this file: System Settings -> Privacy & Security
# -> 'Open Anyway'. Or run in Terminal:  sh preview-mac.command
cd "$(dirname "$0")"
PORT=8123
(sleep 1; open "http://127.0.0.1:$PORT") &
if command -v ruby >/dev/null 2>&1 && ruby -e 'require "webrick"' >/dev/null 2>&1; then
  exec ruby -run -e httpd . -p $PORT
elif command -v python3 >/dev/null 2>&1; then
  exec python3 -m http.server $PORT --bind 127.0.0.1
elif command -v php >/dev/null 2>&1; then
  exec php -S 127.0.0.1:$PORT
else
  echo "No ruby/python3/php found - please contact the site author."; read -r _
fi