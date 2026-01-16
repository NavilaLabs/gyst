#!/bin/bash

# Make scripts executable
chmod +x /workspaces/loom/scripts/*.sh

# Add scripts to PATH if not already there
if [[ ":$PATH:" != *":/workspaces/loom/scripts:"* ]]; then
    echo 'export PATH="/workspaces/loom/scripts:$PATH"' >> ~/.bashrc
fi

# Install Deno only if it's missing, or use -f to force/overwrite
if ! command -v deno &> /dev/null; then
    curl -fsSL https://deno.land/install.sh | sh
else
    echo "Deno already installed, skipping..."
fi

# Ensure Deno binaries are in the current subshell PATH for the next command
export PATH="$HOME/.deno/bin:$PATH"

# Install global packages (these usually handle "already installed" gracefully)
deno install -g npm:tailwind npm:@tailwindcss/cli
