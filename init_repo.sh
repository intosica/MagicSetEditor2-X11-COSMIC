#!/usr/bin/env bash
set -euo pipefail

REMOTE="https://github.com/intosica/MagicSetEditor2-X11-COSMIC"

git init -b main
git remote add origin "$REMOTE" 2>/dev/null || git remote set-url origin "$REMOTE"

# Optional: if the GitHub repo already has commits (README/license), integrate them first:
#   git pull origin main --allow-unrelated-histories

git add .
git commit -m "Initial workspace: mse_core skeleton + libcosmic split-pane app"
git push -u origin main
