#!/bin/sh

# Set the secret if not already provided
if [ -z "$ZK_AUTHENTICATION_SECRET" ]; then
  echo "ERROR: ZK_AUTHENTICATION_SECRET is not set. Aborting." >&2
  exit 1
fi

# Start the daemons in the background
# echo "Starting Admin Projection Daemon..."
./admin-projection-daemon &

# echo "Starting Tenant Projection Daemon..."
./tenant-projection-daemon &

# Start the main server in the foreground
# 'exec' makes this process PID 1, which is important for Docker
echo "Starting Main Server..."
exec ./server
