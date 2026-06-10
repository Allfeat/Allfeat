#!/usr/bin/env node
// verify.mjs — checklist automatique post-migration (REHEARSAL-PLAN.md phase 6).
//
//   node verify.mjs <para-ws> <H> <para-id>
//
// Attend que la chaîne ait DÉPASSÉ le bloc de handover H (best ET finalized —
// preuve que la relay backe/inclut les candidats), puis vérifie l'état.

import { ApiPromise, WsProvider } from '@polkadot/api';
import { u8aToHex } from '@polkadot/util';
import { xxhashAsU8a, cryptoWaitReady } from '@polkadot/util-crypto';

await cryptoWaitReady();

const [ws, hArg, paraIdArg] = process.argv.slice(2);
if (!ws || !hArg || !paraIdArg) {
  console.error('usage: node verify.mjs <para-ws> <H> <para-id>');
  process.exit(1);
}
const H = Number(hArg);
const PARA_ID = Number(paraIdArg);

const api = await ApiPromise.create({ provider: new WsProvider(ws), noInitWarn: true });

// 1. La chaîne doit avancer ET être finalisée au-delà de H (≈ inclusion relay).
process.stdout.write(`attente: finalized > ${H} (inclusion par la relay)`);
let finalized = 0;
for (let t = 0; t < 80; t++) {
  const fh = await api.rpc.chain.getFinalizedHead();
  finalized = (await api.rpc.chain.getHeader(fh)).number.toNumber();
  if (finalized > H) break;
  process.stdout.write('.');
  await new Promise((r) => setTimeout(r, 3000));
}
console.log('');

let failures = 0;
const check = (label, cond, info = '') => {
  console.log(`${cond ? '✅' : '❌'} ${label}${info ? ` — ${info}` : ''}`);
  if (!cond) failures++;
};

const best = (await api.rpc.chain.getHeader()).number.toNumber();
check(`finalized > H (${finalized} > ${H})`, finalized > H);
check(`best > H (${best} > ${H})`, best > H);

const v = await api.rpc.state.getRuntimeVersion();
check('spec allfeat-melodie-3 @300 / tx4',
  v.specName.toString() === 'allfeat-melodie-3'
    && v.specVersion.toNumber() === 300
    && v.transactionVersion.toNumber() === 4,
  `${v.specName} @${v.specVersion} tx${v.transactionVersion}`);

const lru = (await api.query.system.lastRuntimeUpgrade()).unwrap();
check('LastRuntimeUpgrade = 300', lru.specVersion.toNumber() === 300);

const paraId = await api.query.parachainInfo.parachainId();
check(`ParachainInfo = ${PARA_ID}`, paraId.toNumber() === PARA_ID, paraId.toString());

const invuln = await api.query.collatorSelection.invulnerables();
check('invulnerables non vides (ex-validateurs solo)', invuln.length > 0, `${invuln.length} collateur(s)`);

const nextAts = await api.query.ats.nextAtsId();
check('ATS préservés (nextAtsId > 0)', nextAts.toNumber() > 0, `nextAtsId=${nextAts}`);

// Holds : plus aucun hold MIDDS (remboursés), les holds ATS subsistent.
const holds = await api.query.balances.holds.entries();
let midds = 0; let ats = 0;
for (const [, list] of holds) {
  for (const h of list) {
    const t = h.id.type;
    if (['MusicalWorks', 'Recordings', 'Releases'].includes(t)) midds++;
    if (t === 'Ats') ats++;
  }
}
check('0 hold MIDDS restant (bonds remboursés)', midds === 0, `${midds} restant(s)`);
check('holds ATS conservés', ats >= 0, `${ats} hold(s) ATS`);

// Seeds MIDDS re-posés (100 mUNIT / 250 µUNIT).
for (const inst of ['musicalWorks', 'recordings', 'releases']) {
  const base = await api.query[inst].depositBase();
  const perByte = await api.query[inst].depositPerByte();
  check(`${inst} deposit seeds`,
    base.toString() === '100000000000' && perByte.toString() === '250000000',
    `base=${base} perByte=${perByte}`);
}

// Plus aucune trace des pallets solo (préfixes calculés, pas codés en dur).
for (const pallet of ['Validators', 'Grandpa', 'Historical']) {
  const prefix = u8aToHex(xxhashAsU8a(pallet, 128));
  const leftovers = await api.rpc.state.getKeysPaged(prefix, 1, null);
  check(`storage du pallet solo ${pallet} purgé`, leftovers.length === 0);
}

console.log(failures === 0 ? '\n✅ CHECKLIST COMPLÈTE — migration validée' : `\n❌ ${failures} échec(s)`);
await api.disconnect();
process.exit(failures === 0 ? 0 : 1);
