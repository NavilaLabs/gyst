#!/bin/bash
set -e

echo "Adding component..."

cd /workspaces/zeitrak/zeitrak-presentation/gui/packages/ui
dx components add "$1" --module-path /workspaces/zeitrak/zeitrak-presentation/gui/packages/ui/src/components/atoms

echo "Component $1 added."
