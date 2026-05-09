#!/bin/bash
set -e

echo "Starting development server..."

cd /workspaces/zeitrak/zeitrak-presentation/gui/packages/web
dx serve --fullstack --port 8080 --addr 0.0.0.0
