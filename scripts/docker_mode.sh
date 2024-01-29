#!/bin/sh

case "$MODE" in
  "dev")
    exec allfeat --dev --tmp
    ;;
  "dev-permanent")
      exec allfeat --dev
      ;;
  "testnet")
    exec allfeat --chain harmonie-live -d /data --name MyContainerNode --rpc-external --ws-external --rpc-cors all
    ;;
  *)
    echo "Unknown mode. Starting with default mode \"testnet\"..."
    exec allfeat --chain harmonie-live -d /data --name MyContainerNode --rpc-external --ws-external --rpc-cors all
    ;;
esac