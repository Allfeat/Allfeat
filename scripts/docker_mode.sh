#!/bin/sh

case "$MODE" in
  "dev")
    # Start the dev local network without permanent storage (reset on restart)
    exec allfeat --dev --tmp
    ;;
  "dev-permanent")
      # Start the dev local network with permanent storage
      exec allfeat --dev
      ;;
  "testnet")
    # Commande pour le mode harmonie
    exec allfeat --chain harmonie --alice -d /data --name MyContainerNode --rpc-external --ws-external --rpc-cors all
    ;;
  *)
    # Commande par défaut si aucune ou une valeur inconnue est fournie pour MODE
    echo "Unknown mode. Starting with default mode \"testnet\"..."
    exec allfeat --chain harmonie --alice -d /data --name MyContainerNode --rpc-external --ws-external --rpc-cors all
    ;;
esac