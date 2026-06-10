#!/usr/bin/env node
// cutover.mjs — Phases 3-4 of REHEARSAL-PLAN.md
//
//   node cutover.mjs set-code    <solo-ws> <wasm-path>
//       sudo.sudoUncheckedWeight(system.setCode(wasm)) signed by //Alice.
//       Including it HALTS the solo chain: the custom node cannot even
//       instantiate the parachain runtime (missing `storage_proof_size`
//       reclaim host function) — expected log signature:
//       "Cannot create a runtime … ext_storage_proof_size_storage_proof_size_version_1".
//
//   node cutover.mjs export-head <solo-ws> [out.hex]
//       SCALE-encoded header of the current BEST block (= H, the setCode
//       block). Run it BEFORE stopping the halted solo node.
//
//   node cutover.mjs register    <relay-ws> <para-id> <head.hex> <wasm-path>
//       sudo(parasSudoWrapper.sudoScheduleParaInitialize(id, {genesisHead,
//       validationCode, paraKind: true})) on the local relay (sudo = Alice).
//       LOCAL shortcut — the real Paseo flow is registrar.reserve/register
//       + coretime.

import { readFileSync, writeFileSync } from 'node:fs';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { compactToU8a, hexToU8a, u8aConcat, u8aToHex } from '@polkadot/util';
import { blake2AsHex, cryptoWaitReady, xxhashAsU8a } from '@polkadot/util-crypto';

const twox128 = (s) => xxhashAsU8a(s, 128);
const storageKey = (pallet, item) => u8aToHex(u8aConcat(twox128(pallet), twox128(item)));

const [cmd, ...args] = process.argv.slice(2);

const connect = (endpoint) =>
  ApiPromise.create({ provider: new WsProvider(endpoint), noInitWarn: true });

const alice = async () => {
  await cryptoWaitReady();
  return new Keyring({ type: 'sr25519' }).addFromUri('//Alice');
};

const signAndWait = (tx, signer) =>
  new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, dispatchError }) => {
      if (dispatchError) {
        reject(new Error(`dispatch error: ${dispatchError.toString()}`));
      } else if (status.isInBlock) {
        console.log(`included in ${status.asInBlock.toHex()}`);
        resolve();
      }
    }).catch(reject);
  });

if (cmd === 'set-code') {
  const [ws, wasmPath] = args;
  const api = await connect(ws);
  const code = u8aToHex(readFileSync(wasmPath));

  // CRITICAL companion write: the PVF's Aura seal check
  // (`cumulus_pallet_aura_ext::BlockExecutor::verify_and_remove_seal`) reads
  // `AuraExt::Authorities` from the PARENT state, BEFORE executing H+1 — so
  // the runtime migration in H+1 seeds it too late ("Invalid AuRa author
  // index 0 for authorities: []" PVF panic, candidate invalid, collator
  // banned). The solo chain's final block must therefore copy the current
  // `Aura::Authorities` bytes (same encoding) into the `AuraExt` key.
  const auraAuthorities = await api.rpc.state.getStorage(storageKey('Aura', 'Authorities'));
  const authoritiesHex = auraAuthorities?.toHex?.() ?? String(auraAuthorities);
  if (!authoritiesHex || authoritiesHex === '0x' || authoritiesHex === 'null') {
    console.error('could not read Aura::Authorities from the solo chain — aborting');
    process.exit(1);
  }
  console.log(`Aura::Authorities (${(authoritiesHex.length - 2) / 2} bytes) → seeded into AuraExt::Authorities`);
  console.log(`setCode: ${(code.length - 2) / 2} bytes on ${ws}`);

  const tx = api.tx.sudo.sudoUncheckedWeight(
    api.tx.utility.batchAll([
      api.tx.system.setStorage([[storageKey('AuraExt', 'Authorities'), authoritiesHex]]),
      api.tx.system.setCode(code),
    ]),
    { refTime: 0, proofSize: 0 },
  );
  await signAndWait(tx, await alice());
  console.log('setStorage(AuraExt::Authorities) + setCode included — the solo chain halts here (block = H).');
  console.log('expected node logs from now on:');
  console.log('  Cannot create a runtime … ext_storage_proof_size… (nominal)');
  console.log('next: node cutover.mjs export-head ' + ws);
  await api.disconnect();
} else if (cmd === 'export-head') {
  const [ws, out = 'genesis-head.hex'] = args;
  // The halted solo node cannot construct the @300 runtime anymore, so a
  // full ApiPromise init (which fetches the runtime version) FAILS — use
  // raw RPC and SCALE-encode the header by hand. The JSON header's
  // `digest.logs` entries are already SCALE-encoded DigestItems.
  const provider = new WsProvider(ws);
  await provider.isReady;
  const hash = await provider.send('chain_getBlockHash', []); // best = H
  const h = await provider.send('chain_getHeader', [hash]);
  const number = parseInt(h.number, 16);
  const hex = u8aToHex(u8aConcat(
    hexToU8a(h.parentHash),
    compactToU8a(number),
    hexToU8a(h.stateRoot),
    hexToU8a(h.extrinsicsRoot),
    compactToU8a(h.digest.logs.length),
    ...h.digest.logs.map((l) => hexToU8a(l)),
  ));
  // Self-check: the block hash IS the blake2-256 of the full SCALE header
  // (seal included) — if this matches, the encoding is provably correct.
  const computed = blake2AsHex(hexToU8a(hex), 256);
  if (computed !== hash) {
    console.error(`encoding self-check FAILED: blake2(header) ${computed} != ${hash}`);
    process.exit(1);
  }
  writeFileSync(out, hex);
  console.log(`best block #${number} (${hash})`);
  console.log(`SCALE header (${(hex.length - 2) / 2} bytes), blake2 self-check OK -> ${out}`);
  console.log('you can stop the halted solo node now (frees the DB for omni-node).');
  await provider.disconnect();
} else if (cmd === 'register') {
  const [ws, paraId, headPath, wasmPath] = args;
  const api = await connect(ws);
  if (!api.tx.paraSudoWrapper) {
    console.error('paraSudoWrapper missing on this relay — use registrar.reserve/register instead.');
    process.exit(1);
  }
  const id = Number(paraId);
  const genesisHead = readFileSync(headPath, 'utf8').trim();
  const validationCode = u8aToHex(readFileSync(wasmPath));
  const signer = await alice();

  // Registration as a full parachain + ONE core: a zombienet network spawned
  // without `[[parachains]]` provisions ZERO cores (`numCores: 0`), and a
  // para without a core never gets a collation slot.
  await signAndWait(
    api.tx.sudo.sudo(api.tx.utility.batchAll([
      api.tx.configuration.setCoretimeCores(1),
      api.tx.paraSudoWrapper.sudoScheduleParaInitialize(id, {
        genesisHead,
        validationCode,
        paraKind: true,
      }),
    ])),
    signer,
  );
  console.log(`para ${id} + setCoretimeCores(1) scheduled.`);

  // The configuration change only activates at a later session boundary.
  for (let i = 0; ; i++) {
    const cfg = (await api.query.configuration.activeConfig()).toJSON();
    if (cfg.schedulerParams.numCores >= 1) break;
    if (i >= 40) {
      console.error('core count never activated — inspect configuration.activeConfig()');
      process.exit(1);
    }
    console.log('waiting for the core count to activate…');
    await new Promise((r) => setTimeout(r, 6000));
  }

  // Pin core 0 to the para, full time (57600/57600 parts).
  await signAndWait(
    api.tx.sudo.sudo(api.tx.coretime.assignCore(0, 0, [[{ Task: id }, 57600]], null)),
    signer,
  );
  console.log(`core 0 assigned to para ${id}.`);

  // Wait until the para is fully onboarded with OUR head before handing over
  // to omni-node.
  for (let i = 0; ; i++) {
    const lc = (await api.query.paras.paraLifecycles(id)).toString();
    const head = await api.query.paras.heads(id);
    const headOk = head.isSome && head.unwrap().toHex() === genesisHead;
    if (lc === 'Parachain' && headOk) break;
    if (i >= 60) {
      console.error(`onboarding timeout (lifecycle=${lc}, head ok=${headOk})`);
      process.exit(1);
    }
    console.log(`waiting for onboarding… (lifecycle=${lc || 'None'})`);
    await new Promise((r) => setTimeout(r, 6000));
  }
  console.log(`para ${id} is a Parachain with the exported head — start omni-node (phase 5/7).`);
  await api.disconnect();
} else {
  console.error('usage:');
  console.error('  node cutover.mjs set-code    <solo-ws> <wasm-path>');
  console.error('  node cutover.mjs export-head <solo-ws> [out.hex]');
  console.error('  node cutover.mjs register    <relay-ws> <para-id> <head.hex> <wasm-path>');
  process.exit(1);
}
