#!/bin/bash
set -e

# Default arguments for production mode
DEFAULT_ARGS="--base-path /data --rpc-external --rpc-cors all --rpc-methods safe --database paritydb"

# Development mode arguments
DEV_ARGS="--dev"

# Check if DEV_MODE is enabled
if [ "$DEV_MODE" = "true" ]; then
  echo "Starting Allfeat node in DEVELOPMENT mode..."
  echo "WARNING: This mode is not secure and should only be used for development!"

  # Use dev arguments if no custom arguments provided
  if [ $# -eq 0 ]; then
    exec /usr/local/bin/allfeat $DEV_ARGS
  else
    # Use custom arguments in dev mode
    exec /usr/local/bin/allfeat --dev "$@"
  fi
else
  echo "Starting Allfeat node in PRODUCTION mode..."

  # Use production arguments if no custom arguments provided
  if [ $# -eq 0 ]; then
    exec /usr/local/bin/allfeat $DEFAULT_ARGS
  else
    # Use custom arguments in production mode
    exec /usr/local/bin/allfeat "$@"
  fi
fi
