#!/usr/bin/env bash

set -Eeuo pipefail
IFS=$'\n\t'

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

BENCHER="${BENCHER_PATH:-frame-omni-bencher}"
PROFILE="${PROFILE:-release}"
SKIP_BUILD="${SKIP_BUILD:-0}"
GENESIS_PRESET="${GENESIS_PRESET:-development}"
STEPS="${STEPS:-50}"
REPEAT="${REPEAT:-20}"
HEAP_PAGES="${HEAP_PAGES:-4096}"
WASM_EXECUTION="${WASM_EXECUTION:-compiled}"

RUNTIME_PACKAGE="melodie-runtime"
RUNTIME_WASM="${ROOT_DIR}/target/${PROFILE}/wbuild/${RUNTIME_PACKAGE}/melodie_runtime.compact.compressed.wasm"
TEMPLATE="${ROOT_DIR}/.maintain/frame-weight-template.hbs"
HEADER_FILE="${ROOT_DIR}/HEADER"
BENCHMARKS_FILE="${ROOT_DIR}/runtime/melodie/src/benchmarks.rs"

RUN_ID="$(date +"%Y%m%d_%H%M%S")"
LOG_DIR="${ROOT_DIR}/target/weight-logs"
LOG_FILE="${LOG_DIR}/testnet_weights_${RUN_ID}.log"
GENERATED_DIR="${LOG_DIR}/generated/testnet/${RUN_ID}"

timestamp() {
  date +"%Y-%m-%d %H:%M:%S"
}

log() {
  printf "[%s] [INFO] %s\n" "$(timestamp)" "$*" | tee -a "${LOG_FILE}"
}

warn() {
  printf "[%s] [WARN] %s\n" "$(timestamp)" "$*" | tee -a "${LOG_FILE}" >&2
}

fail() {
  printf "[%s] [ERROR] %s\n" "$(timestamp)" "$*" | tee -a "${LOG_FILE}" >&2
  exit 1
}

on_error() {
  local line="$1"
  local cmd="$2"
  fail "Unexpected failure at line ${line}: ${cmd}"
}

trap 'on_error "${LINENO}" "${BASH_COMMAND}"' ERR

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing command: $1"
}

resolve_pallet_name() {
  local preferred="$1"
  local dashed
  dashed="${preferred//_/-}"

  if printf "%s\n" "${AVAILABLE_PALLETS}" | grep -Fxq "${preferred}"; then
    printf "%s\n" "${preferred}"
    return 0
  fi

  if printf "%s\n" "${AVAILABLE_PALLETS}" | grep -Fxq "${dashed}"; then
    printf "%s\n" "${dashed}"
    return 0
  fi

  return 1
}

output_path_for_pallet() {
  local pallet="$1"
  case "${pallet}" in
    pallet_validators) echo "${ROOT_DIR}/pallets/validators/src/weights.rs" ;;
    pallet_ats) echo "${ROOT_DIR}/pallets/ats/src/weights.rs" ;;
    pallet_midds) echo "${ROOT_DIR}/pallets/midds/src/weights.rs" ;;
    *) echo "${GENERATED_DIR}/${pallet}.rs" ;;
  esac
}

mkdir -p "${LOG_DIR}"
: >"${LOG_FILE}"

log "Generating TESTNET weights (Melodie)"
log "Log: ${LOG_FILE}"
log "Config: PROFILE=${PROFILE}, GENESIS_PRESET=${GENESIS_PRESET}, STEPS=${STEPS}, REPEAT=${REPEAT}, HEAP_PAGES=${HEAP_PAGES}, WASM_EXECUTION=${WASM_EXECUTION}, SKIP_BUILD=${SKIP_BUILD}"

require_cmd cargo
require_cmd "${BENCHER}"

if [[ "${SKIP_BUILD}" != "1" ]]; then
  log "Build runtime ${RUNTIME_PACKAGE} (feature runtime-benchmarks)"
  cargo build \
    --profile "${PROFILE}" \
    --package "${RUNTIME_PACKAGE}" \
    --features runtime-benchmarks >>"${LOG_FILE}" 2>&1
else
  warn "SKIP_BUILD=1: build skipped"
fi

[[ -f "${RUNTIME_WASM}" ]] || fail "WASM not found: ${RUNTIME_WASM}"
[[ -f "${TEMPLATE}" ]] || fail "Template not found: ${TEMPLATE}"
[[ -f "${HEADER_FILE}" ]] || fail "Header not found: ${HEADER_FILE}"
[[ -f "${BENCHMARKS_FILE}" ]] || fail "Benchmark file not found: ${BENCHMARKS_FILE}"

log "Fetching list of benchmarkable pallets"
AVAILABLE_PALLETS="$("${BENCHER}" v1 benchmark pallet \
  --runtime "${RUNTIME_WASM}" \
  --list=pallets \
  --no-csv-header \
  --genesis-builder-preset="${GENESIS_PRESET}" 2>>"${LOG_FILE}")"

mapfile -t TARGET_PALLETS < <(sed -n 's/^[[:space:]]*\[\([^,[:space:]]\+\),.*/\1/p' "${BENCHMARKS_FILE}")
[[ "${#TARGET_PALLETS[@]}" -gt 0 ]] || fail "No pallets parsed from ${BENCHMARKS_FILE}"

success_count=0
fail_count=0
start_epoch="$(date +%s)"

for pallet in "${TARGET_PALLETS[@]}"; do
  output_file="$(output_path_for_pallet "${pallet}")"
  output_rel="${output_file#${ROOT_DIR}/}"
  if [[ "${output_file}" == "${GENERATED_DIR}"/* ]]; then
    output_rel="target/weight-logs/generated/testnet/${RUN_ID}/$(basename -- "${output_file}")"
  fi
  mkdir -p "$(dirname -- "${output_file}")"

  resolved_pallet="$(resolve_pallet_name "${pallet}")" || {
    warn "Pallet missing from benchmark list: ${pallet}"
    fail_count=$((fail_count + 1))
    continue
  }

  log "Benchmark ${resolved_pallet} -> ${output_rel}"
  if "${BENCHER}" v1 benchmark pallet \
    --runtime "${RUNTIME_WASM}" \
    --genesis-builder-preset="${GENESIS_PRESET}" \
    --pallet="${resolved_pallet}" \
    --extrinsic="*" \
    --steps="${STEPS}" \
    --repeat="${REPEAT}" \
    --wasm-execution="${WASM_EXECUTION}" \
    --heap-pages="${HEAP_PAGES}" \
    --header="${HEADER_FILE}" \
    --template="${TEMPLATE}" \
    --output="${output_file}" >>"${LOG_FILE}" 2>&1; then
    success_count=$((success_count + 1))
    log "OK ${resolved_pallet}"
  else
    fail_count=$((fail_count + 1))
    warn "FAILED ${resolved_pallet}"
  fi
done

elapsed="$(( $(date +%s) - start_epoch ))"
log "TESTNET summary: success=${success_count}, failed=${fail_count}, total=${#TARGET_PALLETS[@]}, duration=${elapsed}s"

if [[ "${fail_count}" -gt 0 ]]; then
  fail "Testnet weight generation failed. See ${LOG_FILE}"
fi

log "Testnet weight generation completed successfully."
