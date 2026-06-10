#!/usr/bin/env node
// fork-melodie.mjs — Phase 1 of REHEARSAL-PLAN.md
//
// Forks the LIVE Melodie solo chain into a locally-authorable chain spec:
// exports the full state at the latest finalized block, keeps everything
// (ATS, MIDDS, balances, holds, `:code` @live) and patches ONLY what is
// needed to take over locally with the well-known //Alice dev key:
//   - Sudo.Key, Aura/Grandpa authorities, pallet-validators set,
//     Session {Validators, NextKeys (OLD {grandpa,aura} 64-byte format),
//     QueuedKeys, KeyOwner} -> Alice
//   - System.* dropped except System.Account + System.LastRuntimeUpgrade
//     (so the later `sudo.setCode(wasm@300)` triggers 201 -> 300 exactly
//     like on prod)
//   - Alice endowed (TotalIssuance adjusted accordingly)
//
// Usage:
//   npm install            (once, in this directory)
//   node fork-melodie.mjs wss://<RPC-MELODIE> [../melodie3-fork.json]

import { writeFileSync } from 'node:fs';
import { ApiPromise, WsProvider } from '@polkadot/api';
import {
  compactToU8a,
  hexToU8a,
  stringToU8a,
  u8aConcat,
  u8aToHex,
} from '@polkadot/util';
import {
  blake2AsHex,
  blake2AsU8a,
  cryptoWaitReady,
  encodeAddress,
  xxhashAsU8a,
} from '@polkadot/util-crypto';

// ---------------------------------------------------------------- helpers
const twox128 = (s) => xxhashAsU8a(s, 128);
const palletItem = (p, i) => u8aToHex(u8aConcat(twox128(p), twox128(i)));
const twox64cat = (b) => u8aConcat(xxhashAsU8a(b, 64), b);
const blake128cat = (b) => u8aConcat(blake2AsU8a(b, 128), b);
const leEncode = (value, bytes) => {
  const out = new Uint8Array(bytes);
  let v = BigInt(value);
  for (let i = 0; i < bytes; i++) {
    out[i] = Number(v & 0xffn);
    v >>= 8n;
  }
  return out;
};
const u32le = (v) => leEncode(v, 4);
const u64le = (v) => leEncode(v, 8);
const u128le = (v) => leEncode(v, 16);
const vec = (...items) => u8aConcat(compactToU8a(items.length), ...items);
const chunks = (arr, n) =>
  Array.from({ length: Math.ceil(arr.length / n) }, (_, i) => arr.slice(i * n, (i + 1) * n));

// ------------------------------------------------------------- constants
// Well-known //Alice dev keys (sr25519 public == AccountId; ed25519 for the
// old grandpa session slot).
const ALICE_ACC = hexToU8a('0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d');
const ALICE_ED = hexToU8a('0x88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee');
const ENDOWMENT = 1_000_000n * 10n ** 12n; // 1M MEL
const NEW_LOGIC_FLAG = 1n << 127n; // pallet-balances AccountData.flags

const K = {
  system: u8aToHex(twox128('System')),
  systemAccount: palletItem('System', 'Account'),
  lastRuntimeUpgrade: palletItem('System', 'LastRuntimeUpgrade'),
  sudoKey: palletItem('Sudo', 'Key'),
  auraAuthorities: palletItem('Aura', 'Authorities'),
  grandpaAuthorities: palletItem('Grandpa', 'Authorities'),
  validatorsValidators: palletItem('Validators', 'Validators'),
  sessionValidators: palletItem('Session', 'Validators'),
  sessionNextKeys: palletItem('Session', 'NextKeys'),
  sessionQueuedKeys: palletItem('Session', 'QueuedKeys'),
  sessionKeyOwner: palletItem('Session', 'KeyOwner'),
  totalIssuance: palletItem('Balances', 'TotalIssuance'),
  atsRegistry: palletItem('Ats', 'AtsRegistry'),
  code: '0x3a636f6465',
};

// ------------------------------------------------------------------ main
const endpoint = process.argv[2];
const outPath = process.argv[3] ?? '../melodie3-fork.json';
if (!endpoint) {
  console.error('usage: node fork-melodie.mjs <ws-endpoint> [out.json]');
  process.exit(1);
}

await cryptoWaitReady();
const provider = new WsProvider(endpoint);
const api = await ApiPromise.create({ provider, noInitWarn: true });

const at = (await api.rpc.chain.getFinalizedHead()).toHex();
const header = await api.rpc.chain.getHeader(at);
const version = await api.rpc.state.getRuntimeVersion(at);
console.log(`forking ${version.specName} @ specVersion ${version.specVersion}`);
console.log(`  block #${header.number.toNumber()} (${at})`);

// 1. Scrape every key/value pair at `at`. `state_getPairs` in one shot when
// the node exposes unsafe RPCs (local node), paged scrape otherwise.
let top = {};
try {
  const pairs = await provider.send('state_getPairs', ['0x', at]);
  for (const [k, v] of pairs) top[k] = v;
  console.log(`  scraped ${pairs.length} pairs via state_getPairs`);
} catch {
  const keys = [];
  let startKey;
  for (;;) {
    const page = await provider.send('state_getKeysPaged', ['0x', 1000, startKey, at]);
    keys.push(...page);
    if (page.length < 1000) break;
    startKey = page[page.length - 1];
  }
  for (const chunk of chunks(keys, 200)) {
    const values = await Promise.all(
      chunk.map((k) => provider.send('state_getStorage', [k, at])),
    );
    chunk.forEach((k, i) => {
      if (values[i] !== null) top[k] = values[i];
    });
  }
  console.log(`  scraped ${keys.length} keys via paged RPC`);
}
if (!top[K.code]) throw new Error('`:code` missing from scrape — aborting');

// 2. Read values needed for consistent patching, BEFORE filtering.
const aliceAccountKey = u8aToHex(
  u8aConcat(hexToU8a(K.systemAccount), blake128cat(ALICE_ACC)),
);
const decodeU128 = (hex, offset) => {
  const b = hexToU8a(hex).slice(offset, offset + 16);
  let v = 0n;
  for (let i = 15; i >= 0; i--) v = (v << 8n) | BigInt(b[i]);
  return v;
};
// AccountInfo = nonce u32 | consumers u32 | providers u32 | sufficients u32
//             | free u128 | reserved u128 | frozen u128 | flags u128
const oldAlice = top[aliceAccountKey];
const oldAliceNonce = oldAlice ? Number(hexToU8a(oldAlice)[0]) : 0; // nonce LSB is enough for dev keys
const oldAliceFunds = oldAlice ? decodeU128(oldAlice, 16) + decodeU128(oldAlice, 32) : 0n;
const oldIssuance = top[K.totalIssuance] ? decodeU128(top[K.totalIssuance], 0) : 0n;

// 3. Filter: start clean on block-machinery state, replace the authority set.
const filtered = {};
let dropped = 0;
for (const [k, v] of Object.entries(top)) {
  const isSystem = k.startsWith(K.system);
  const keepSystem = k.startsWith(K.systemAccount) || k === K.lastRuntimeUpgrade;
  if (isSystem && !keepSystem) { dropped++; continue; }
  if (k.startsWith(K.sessionNextKeys) || k === K.sessionQueuedKeys || k === K.sessionValidators) {
    dropped++; continue;
  }
  filtered[k] = v;
}

// 4. Patch in the Alice takeover. The 64-byte session keys use the OLD solo
// layout `{ grandpa: ed25519, aura: sr25519 }` — field order matters, and it
// is exactly what `TransitionToParachain` re-encodes during the rehearsal.
const oldSessionKeys = u8aConcat(ALICE_ED, ALICE_ACC);
filtered[K.sudoKey] = u8aToHex(ALICE_ACC);
filtered[K.auraAuthorities] = u8aToHex(vec(ALICE_ACC));
filtered[K.grandpaAuthorities] = u8aToHex(vec(u8aConcat(ALICE_ED, u64le(1n))));
filtered[K.validatorsValidators] = u8aToHex(vec(ALICE_ACC));
filtered[K.sessionValidators] = u8aToHex(vec(ALICE_ACC));
filtered[K.sessionQueuedKeys] = u8aToHex(vec(u8aConcat(ALICE_ACC, oldSessionKeys)));
filtered[u8aToHex(u8aConcat(hexToU8a(K.sessionNextKeys), twox64cat(ALICE_ACC)))] =
  u8aToHex(oldSessionKeys);
const keyOwnerKey = (keyType, pub) =>
  u8aToHex(u8aConcat(
    hexToU8a(K.sessionKeyOwner),
    twox64cat(u8aConcat(stringToU8a(keyType), compactToU8a(pub.length), pub)),
  ));
filtered[keyOwnerKey('gran', ALICE_ED)] = u8aToHex(ALICE_ACC);
filtered[keyOwnerKey('aura', ALICE_ACC)] = u8aToHex(ALICE_ACC);
// Alice: consumers=1 (session keys), providers=1 (free >= ED), new-logic flag.
filtered[aliceAccountKey] = u8aToHex(u8aConcat(
  u32le(oldAliceNonce), u32le(1), u32le(1), u32le(0),
  u128le(ENDOWMENT), u128le(0n), u128le(0n), u128le(NEW_LOGIC_FLAG),
));
filtered[K.totalIssuance] = u8aToHex(u128le(oldIssuance - oldAliceFunds + ENDOWMENT));

// 5. Assemble the spec. Plain `GenericChainSpec` (the custom node has no
// required extensions); add `relay_chain`/`para_id` later for omni-node.
const spec = {
  name: 'Melodie 3 Fork (rehearsal)',
  id: 'melodie3_fork',
  chainType: 'Local',
  bootNodes: [],
  telemetryEndpoints: null,
  protocolId: 'melodie3-fork',
  properties: { tokenSymbol: 'MEL', tokenDecimals: 12, ss58Format: 42 },
  codeSubstitutes: {},
  genesis: { raw: { top: filtered, childrenDefault: {} } },
};
writeFileSync(outPath, JSON.stringify(spec, null, 1));

// 6. Traceability summary (compare these anchors again in Phase 6).
const atsCount = Object.keys(filtered).filter((k) => k.startsWith(K.atsRegistry)).length;
const codeHex = filtered[K.code];
console.log('---');
console.log(`kept ${Object.keys(filtered).length} keys (dropped ${dropped})`);
console.log(`:code  ${(codeHex.length - 2) / 2} bytes, blake2-256 ${blake2AsHex(hexToU8a(codeHex), 256)}`);
console.log(`ATS records: ${atsCount}`);
console.log(`alice: ${encodeAddress(ALICE_ACC, 42)} endowed with 1M MEL (sudo + validator)`);
console.log(`spec written to ${outPath}`);
console.log('---');
console.log('next (REHEARSAL-PLAN.md phase 1.4):');
console.log('  ./allfeat --chain melodie3-fork.json --alice --validator \\');
console.log('    --base-path /tmp/rehearsal/solo --database paritydb --rpc-port 9944 \\');
console.log('    --unsafe-force-node-key-generation');

await api.disconnect();
