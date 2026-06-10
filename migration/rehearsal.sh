#!/usr/bin/env bash
# rehearsal.sh — répétition complète de la migration solo→para de Melodie, en
# une commande (REHEARSAL-PLAN.md, phases 1→6 automatisées).
#
#   ./rehearsal.sh run      # tout dérouler : fork → relay → setCode → register
#                           # → omni-node → vérification automatique
#   ./rehearsal.sh verify   # rejouer uniquement la checklist d'état
#   ./rehearsal.sh status   # endpoints + processus en cours
#   ./rehearsal.sh stop     # tout arrêter (relay comprise) et nettoyer les pids
#
# Seule variable OBLIGATOIRE : MELODIE_RPC (endpoint de la solo live).
#   MELODIE_RPC=wss://… ./rehearsal.sh run

set -euo pipefail

###############################################################################
#                         CONFIG — variables d'environnement                  #
###############################################################################
MELODIE_RPC="${MELODIE_RPC:-}"          # RPC de la Melodie LIVE (obligatoire)
PARA_ID="${PARA_ID:-2000}"
SOLO_RPC_PORT="${SOLO_RPC_PORT:-9944}"
PARA_RPC_PORT="${PARA_RPC_PORT:-9988}"
BASE_PATH="${BASE_PATH:-/tmp/rehearsal/solo}"
RUN_DIR="${RUN_DIR:-/tmp/rehearsal/run}"
###############################################################################

ROOT="$(cd "$(dirname "$0")" && pwd)"               # migration/
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
ALLFEAT_BIN="${ALLFEAT_BIN:-$REPO_ROOT/target/release/allfeat}"
WASM="${WASM:-$REPO_ROOT/target/release/wbuild/melodie-runtime/melodie_runtime.compact.compressed.wasm}"
FORK_SPEC="$ROOT/melodie3-fork.json"
PARA_SPEC="$ROOT/melodie3-fork-para.json"
RELAY_SPEC="$ROOT/paseo-local-relay.json"
HEAD_HEX="$ROOT/genesis-head.hex"
RELAY_WS=""
mkdir -p "$RUN_DIR"

step()  { printf '\n\033[1;36m▶ %s\033[0m\n' "$*"; }
ok()    { printf '\033[1;32m  ✓ %s\033[0m\n' "$*"; }
die()   { printf '\033[1;31m  ✗ %s\033[0m\n' "$*" >&2; exit 1; }

rpc() { # rpc <port> <method> [params-json]
  curl -s -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$2\",\"params\":${3:-[]}}" \
    "http://127.0.0.1:$1" | jq -r '.result // empty'
}

best_number() { # best_number <port>  → numéro de bloc décimal (vide si down)
  local h; h=$(rpc "$1" chain_getHeader | jq -r '.number // empty' 2>/dev/null) || return 0
  [ -n "$h" ] && printf '%d' "$h" 2>/dev/null || true
}

wait_for() { # wait_for <desc> <timeout_s> <cmd…>  (la cmd doit réussir ET sortir non-vide)
  local desc="$1" timeout="$2"; shift 2
  local t=0
  while [ "$t" -lt "$timeout" ]; do
    if out=$("$@" 2>/dev/null) && [ -n "$out" ]; then ok "$desc"; return 0; fi
    sleep 2; t=$((t + 2))
  done
  die "timeout (${timeout}s) : $desc"
}

newest_zombie_dir() { ls -td /var/folders/*/*/T/zombie-* /tmp/zombie-* 2>/dev/null | head -1; }

discover_relay() { # remplit RELAY_WS + RELAY_SPEC + bootnodes si une relay tourne
  local z; z=$(newest_zombie_dir); [ -n "$z" ] && [ -f "$z/zombie.json" ] || return 1
  local port
  port=$(grep -oE '"ws_uri": *"ws://127.0.0.1:[0-9]+"' "$z/zombie.json" | head -1 | grep -oE '[0-9]+' | tail -1)
  [ -n "$port" ] || return 1
  [ -n "$(rpc "$port" chain_getBlockHash '[0]')" ] || return 1
  RELAY_WS="$port"
  cp "$z/paseo-local.json" "$RELAY_SPEC"
  grep -oE '"/ip4/127.0.0.1/tcp/[0-9]+/ws/p2p/[A-Za-z0-9]+"' "$z/zombie.json" \
    | tr -d '"' | sort -u > "$RUN_DIR/bootnodes"
  [ -s "$RUN_DIR/bootnodes" ] || return 1
  return 0
}

cmd_stop() {
  step "Arrêt de tout le réseau de répétition"
  for p in omni solo relay; do
    if [ -f "$RUN_DIR/$p.pid" ]; then
      kill "$(cat "$RUN_DIR/$p.pid")" 2>/dev/null || true
      rm -f "$RUN_DIR/$p.pid"
      ok "$p arrêté"
    fi
  done
  pkill -f '/T/zombie-' 2>/dev/null || true   # validateurs relay résiduels
  ok "processus zombienet purgés"
}

cmd_status() {
  if discover_relay >/dev/null 2>&1; then
    echo "relay  : ws://127.0.0.1:$RELAY_WS (up)"
  else
    echo "relay  : down"
  fi
  echo "solo   : port $SOLO_RPC_PORT  best=$(best_number "$SOLO_RPC_PORT")"
  echo "para   : port $PARA_RPC_PORT  best=$(best_number "$PARA_RPC_PORT")"
  echo "logs   : $RUN_DIR/{relay,solo,omni}.log"
}

cmd_verify() {
  [ -f "$RUN_DIR/H" ] || die "pas de H enregistré ($RUN_DIR/H) — lancer ./rehearsal.sh run d'abord"
  (cd "$ROOT/scripts" && node verify.mjs "ws://127.0.0.1:$PARA_RPC_PORT" "$(cat "$RUN_DIR/H")" "$PARA_ID")
}

cmd_run() {
  step "0/8 Pré-vol"
  for b in node jq pop polkadot-omni-node curl; do
    command -v "$b" >/dev/null || die "binaire manquant : $b"
  done
  [ -x "$ALLFEAT_BIN" ] || die "node custom introuvable : $ALLFEAT_BIN (le crate node/ a été retiré du repo : utiliser un binaire de release existant ou builder depuis l'historique git pré-migration)"
  [ -f "$WASM" ] || die "wasm @300 introuvable : $WASM (cargo build --release à la racine du repo)"
  [ -n "$MELODIE_RPC" ] || die "MELODIE_RPC non défini (export MELODIE_RPC=wss://…)"
  [ -d "$ROOT/scripts/node_modules" ] || (cd "$ROOT/scripts" && npm install --no-fund --no-audit --silent)
  mkdir -p "$RUN_DIR"
  ok "outillage complet"

  step "1/8 Arrêt des restes éventuels + base-path vierge"
  cmd_stop >/dev/null 2>&1 || true
  rm -rf "$BASE_PATH"
  ok "état nettoyé"

  step "2/8 Fork de la Melodie live ($MELODIE_RPC)"
  (cd "$ROOT/scripts" && node fork-melodie.mjs "$MELODIE_RPC" "$FORK_SPEC") | tee "$RUN_DIR/fork.log"

  step "3/8 Relay paseo-local"
  if discover_relay; then
    ok "relay déjà en marche réutilisée (ws:$RELAY_WS)"
  else
    (cd "$ROOT" && nohup pop up -y ./network-rehearsal.toml > "$RUN_DIR/relay.log" 2>&1 & echo $! > "$RUN_DIR/relay.pid")
    wait_for "réseau relay découvert et joignable" 240 "$0" __discover
    discover_relay || die "relay introuvable après spawn"
    ok "relay lancée (ws:$RELAY_WS)"
  fi
  wait_for "relay produit des blocs" 120 bash -c "n=\$('$0' __best $RELAY_WS); [ -n \"\$n\" ] && [ \"\$n\" -ge 1 ] && echo \$n"

  step "4/8 Solo forkée (node custom)"
  nohup "$ALLFEAT_BIN" --chain "$FORK_SPEC" --alice --validator \
    --base-path "$BASE_PATH" --database paritydb --rpc-port "$SOLO_RPC_PORT" \
    --unsafe-force-node-key-generation > "$RUN_DIR/solo.log" 2>&1 &
  echo $! > "$RUN_DIR/solo.pid"
  wait_for "solo forkée produit des blocs" 120 bash -c "n=\$('$0' __best $SOLO_RPC_PORT); [ -n \"\$n\" ] && [ \"\$n\" -ge 1 ] && echo \$n"

  step "5/8 Cutover : batch setStorage(AuraExt)+setCode, puis export du header"
  (cd "$ROOT/scripts" && node cutover.mjs set-code "ws://127.0.0.1:$SOLO_RPC_PORT" "$WASM") | tee "$RUN_DIR/setcode.log"
  (cd "$ROOT/scripts" && node cutover.mjs export-head "ws://127.0.0.1:$SOLO_RPC_PORT" "$HEAD_HEX") | tee "$RUN_DIR/head.log"
  H=$(grep -oE 'best block #[0-9]+' "$RUN_DIR/head.log" | grep -oE '[0-9]+')
  [ -n "$H" ] || die "impossible d'extraire H"
  echo "$H" > "$RUN_DIR/H"
  kill "$(cat "$RUN_DIR/solo.pid")" 2>/dev/null || true; rm -f "$RUN_DIR/solo.pid"
  ok "solo arrêtée à H=$H, header exporté ($HEAD_HEX)"

  step "6/8 Registration du para $PARA_ID (cores + paraInitialize + assignCore)"
  (cd "$ROOT/scripts" && node cutover.mjs register "ws://127.0.0.1:$RELAY_WS" "$PARA_ID" "$HEAD_HEX" "$WASM") | tee "$RUN_DIR/register.log"

  step "7/8 Omni-node collateur sur la DB de la solo"
  jq ". + {relay_chain: \"paseo-local\", para_id: $PARA_ID}" "$FORK_SPEC" > "$PARA_SPEC"
  BOOTNODE_FLAGS=""
  while IFS= read -r bn; do BOOTNODE_FLAGS="$BOOTNODE_FLAGS --bootnodes $bn"; done < "$RUN_DIR/bootnodes"
  # shellcheck disable=SC2086
  nohup polkadot-omni-node --chain "$PARA_SPEC" --collator --alice \
    --base-path "$BASE_PATH" --database paritydb \
    --relay-chain-rpc-urls "ws://127.0.0.1:$RELAY_WS" --rpc-port "$PARA_RPC_PORT" \
    -- --chain "$RELAY_SPEC" --network-backend libp2p $BOOTNODE_FLAGS \
    > "$RUN_DIR/omni.log" 2>&1 &
  echo $! > "$RUN_DIR/omni.pid"
  ok "omni-node lancé (logs: $RUN_DIR/omni.log)"

  step "8/8 Vérification automatique post-migration"
  cmd_verify

  printf '\n\033[1;32m🎉 Répétition complète RÉUSSIE.\033[0m\n'
  echo "   para RPC : ws://127.0.0.1:$PARA_RPC_PORT   relay RPC : ws://127.0.0.1:$RELAY_WS"
  echo "   logs     : $RUN_DIR/"
  echo "   tout arrêter : $0 stop"
}

case "${1:-}" in
  run)        cmd_run ;;
  stop)       cmd_stop ;;
  status)     cmd_status ;;
  verify)     cmd_verify ;;
  __best)     best_number "$2" ;;                    # interne (wait_for)
  __discover) mkdir -p "$RUN_DIR"; discover_relay && echo ok ;;  # interne
  *) echo "usage: $0 {run|verify|status|stop}   (MELODIE_RPC=wss://… $0 run)"; exit 1 ;;
esac
