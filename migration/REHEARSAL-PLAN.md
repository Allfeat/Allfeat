# Répétition locale — fork de Melodie live → parachain sur Paseo local

## ⚡ TL;DR — une commande

```sh
# pré-requis one-shot : binaire du node solo pré-migration (le crate node/ a été
# retiré du repo — release existante ou build depuis l'historique git) + wasm @300
# (`cargo build --release` à la racine du repo).
# Toutes les commandes de ce document s'exécutent depuis migration/.
MELODIE_RPC=wss://<RPC-MELODIE-LIVE> ./rehearsal.sh run
```

`rehearsal.sh` déroule TOUT (fork → relay pop → setCode batch → export head →
registration+cores → omni-node → **checklist automatique 15 points**) et
laisse le réseau tourner. Autres commandes : `verify` (rejoue la checklist),
`status`, `stop`. Variables optionnelles (défauts sains) : `PARA_ID`,
`BASE_PATH`, `RUN_DIR`, `ALLFEAT_BIN`, `WASM`, `SOLO_RPC_PORT`,
`PARA_RPC_PORT`. Logs et artefacts : `/tmp/rehearsal/run/`.

Le reste de ce document détaille chaque phase (pour comprendre, déboguer, et
transposer au cutover Paseo réel).

---

Objectif : rejouer **toute** la migration solo→para en local, sur l'état réel de
la prod : fork de la solochain Melodie live → batch
`setStorage(AuraExt)+setCode(wasm@300)` (halt) →
enregistrement sur une relay paseo-local → handoff de la DB au
`polkadot-omni-node` → bloc H+1 exécute `TransitionToParachain` → la chaîne
continue avec les ATS intacts.

Ce que cette répétition dérisque, dans l'ordre d'importance :
1. `TransitionToParachain` + migrations ATS v0→v2 sur l'**état réel** (schémas,
   index de `HoldReason`, poids/PoV du bloc H+1) ;
2. l'ouverture de la **DB créée par le node custom** par omni-node ;
3. la mécanique d'enregistrement avec `genesis_head = header(H)` ;
4. le runbook opérationnel (ordre des commandes, signaux d'arrêt, timings).

---

## Phase 0 — Pré-vol : try-runtime sur l'état live (sans infra)

Le test le moins cher et le plus informatif. À faire AVANT de monter le réseau.

```sh
# 0.1 installer try-runtime-cli (une fois)
cargo install --git https://github.com/paritytech/try-runtime-cli --locked

# 0.2 builder le runtime AVEC les hooks try-runtime (à la racine du repo)
cargo build --release --features try-runtime

# 0.3 rejouer l'upgrade sur un snapshot live de la prod
#     --disable-mbm-checks : la phase MBM de l'outil produit un bloc vide
#     SANS l'inherent cumulus `set_validation_data` → panic attendu de
#     parachain-system sur un runtime parachain ; on n'a aucune MBM, on la
#     désactive.
try-runtime \
  --runtime target/release/wbuild/melodie-runtime/melodie_runtime.compact.compressed.wasm \
  on-runtime-upgrade --checks=all --disable-mbm-checks \
  live --uri wss://<RPC-MELODIE-PROD>
```

✅ Critères de succès — VALIDÉ le 2026-06-09 sur la prod (spec **201**, 3252 clés) :
- log `runtime::solo-to-para` : `solo→para transition done: 2 MIDDS bonds
  released (2 legacy hold entries rewritten), ~724 storage ops` — les 2 holds
  MIDDS @201 (anciens indices 102/103/104) décodés en legacy et remboursés,
  plus AUCUNE erreur `(key, value) failed to decode` ;
- idempotence ✓ (2ᵉ passage : `no solo-chain state detected … skipped`,
  storage roots identiques) ;
- PoV **10,6 KiB** / ref_time 2,1 % → multi-block **non nécessaire** ;
- avertissement bénin restant : `Aura declares internal migrations …
  StorageVersion(0) vs NoStorageVersionSet` (pallet-aura sans version
  déclarée — aucun effet).

À re-vérifier si la prod évolue avant le cutover (nouveau spec solo,
nouvelles données) : relancer cette commande, mêmes critères.

Optionnel (équivalent interactif) : chopsticks avec `wasm-override` pointant le
wasm @300 + `dev_newBlock` pour inspecter l'état post-migration à la main.

---

## Phase 1 — Fork local de la Melodie live (node custom)

But : une solochain locale qui démarre avec **l'état réel** de la prod (et son
`:code` live @201 !), mais authorable par Alice.

### 1.1 → 1.3 Outillé : `scripts/fork-melodie.mjs`

```sh
cd scripts && npm install      # une fois
node fork-melodie.mjs wss://<RPC-MELODIE> ../melodie3-fork.json
```

Le script exporte tout l'état au dernier bloc finalisé (`state_getPairs` en un
appel si le RPC expose les méthodes unsafe — nœud local —, sinon scrape pagé),
écrit `melodie3-fork.json` (id `melodie3_fork`, type Local, properties MEL/12)
et imprime les ancres de traçabilité à noter pour la Phase 6 (nombre d'entrées
`AtsRegistry`, hash blake2 du `:code`, bloc source). `System.*` est purgé SAUF
`System.Account` et `System.LastRuntimeUpgrade` — ce dernier pour que le
`setCode` déclenche 201→300 exactement comme en prod.

Il GARDE tout le reste, et remplace uniquement la prise de contrôle locale :

| Clé | Remplacement | Pourquoi |
|---|---|---|
| `Sudo.Key` | Alice | pouvoir faire le setCode local |
| `Aura.Authorities` | `[alice_sr25519]` | authoring local |
| `Grandpa.Authorities` (si présent) | `[alice_ed25519]` | finalité locale (sinon : tourne non finalisé, acceptable) |
| `Validators.Validators` (pallet solo, idx 7) | `[AliceAccount]` | source des invulnerables post-migration → Alice collateur |
| `Session.Validators` | `[AliceAccount]` | cohérence |
| `Session.NextKeys[Alice]` | `{gran: alice_ed, aura: alice_sr}` **encodé à l'ancien format 64o** | c'est CE que la migration ré-encode — garder le format réel |
| `Session.QueuedKeys` | `[(AliceAccount, vieux format)]` | idem |
| `System.Account[Alice]` | solde confortable | frais |

NE PAS touché par le script : `:code` (reste le runtime live @201 — le setCode
fait partie de la répétition), tout l'état ATS/MIDDS/Balances/holds,
`TransactionPayment`. (Le genesis hash du fork diffère de la prod — attendu
pour une répétition.)

⚠️ Chaque re-fork produit un NOUVEAU genesis : toujours repartir d'un
`--base-path` vierge (`rm -rf /tmp/rehearsal/solo`), sinon collision avec la
DB du fork précédent (même id de chaîne, genesis différent).

### 1.4 Lancer la solo forkée avec le **node custom**

```sh
./allfeat --chain melodie3-fork.json --validator --alice \
  --base-path /tmp/rehearsal/solo --database paritydb \
  --rpc-port 9944 \
  --unsafe-force-node-key-generation
```

(`--unsafe-force-node-key-generation` : un nœud `--validator` sur une chaîne
non-dev refuse de générer sa clé réseau libp2p lui-même —
`NetworkKeyNotFound` sinon. Acceptable en répétition locale ; en prod la clé
existe déjà sur les machines. La clé créée ici sous
`chains/melodie3_fork/network/` sera retrouvée telle quelle par omni-node en
Phase 5 — même base-path, même id.)

✅ Critères : blocs produits ; `state_getStorage` retourne des entrées
`AtsRegistry` réelles ; `system_version` = spec live (2xx).

---

## Phase 2 — Relay paseo-local (relay seule, sans para)

`network-rehearsal.toml` (dans `migration/`) — **relay uniquement**,
on n'enregistre PAS la para par zombienet (on le fera à la main avec le header) :

```toml
[relaychain]
chain = "paseo-local"

[[relaychain.nodes]]
name = "alice"
validator = true

[[relaychain.nodes]]
name = "bob"
validator = true
```

```sh
pop up ./network-rehearsal.toml   # (fallback: zombienet spawn si pop exige une section parachain)
```

Noter le port ws d'alice (relay) : `ws://127.0.0.1:<RELAY_WS>`.

---

## Phase 3 — Build @300 et cutover local

```sh
# 3.1 build du runtime cible (PARA_ID=2000 convient en local) — à la racine du repo
cargo build --release --locked     # ou: pop build --release
WASM=../target/release/wbuild/melodie-runtime/melodie_runtime.compact.compressed.wasm

# 3.2 setCode sur la solo forkée (sudo = Alice) :
node scripts/cutover.mjs set-code ws://127.0.0.1:9944 $WASM
```

⚠️ Le `set-code` du script est en réalité un BATCH
`[system.setStorage(AuraExt::Authorities ← copie brute de Aura::Authorities),
system.setCode(wasm)]` — INDISPENSABLE (découvert en répétition) : le PVF
vérifie le seal Aura de H+1 en lisant `AuraExt::Authorities` dans l'état du
PARENT (H), AVANT d'exécuter le bloc et donc avant la migration qui le seed.
Sans cette clé : panic PVF `Invalid AuRa author index 0 for authorities: []`
(cumulus-pallet-aura-ext `verify_and_remove_seal`) chez les validateurs →
candidat invalide → collateur **banni** (`Report <peer-id> … A collator was
reported by another subsystem. Banned` dans les logs validateurs) → côté
collateur, symptôme trompeur `Collation wasn't advertised to any validator`
+ re-propose en boucle depuis H. Les logs VALIDATEURS sont le bon endroit où
regarder ce genre d'échec.

✅ Signal attendu — OBSERVÉ en répétition le 2026-06-10 : la solo S'ARRÊTE au
bloc H (celui qui inclut le setCode). Le node custom logue en boucle :
`Cannot create a runtime … runtime requires function imports which are not
present on the host: 'env:ext_storage_proof_size_storage_proof_size_version_1'`
— il ne peut pas instancier le runtime parachain (host function de
PoV-reclaim absente d'un node solo ; omni-node l'a). Grandpa stalle 2 blocs
derrière (`invalid authorities set`), sans conséquence : la finalité solo
meurt avec la chaîne. **Comportement nominal — ne PAS redémarrer/rollback** ;
c'est la signature de logs à communiquer aux opérateurs pour le jour J.

```sh
# 3.3 récupérer header(H) SCALE — AVANT d'arrêter le node (RPC requis) :
node scripts/cutover.mjs export-head ws://127.0.0.1:9944 genesis-head.hex
# 3.4 arrêter le process du node custom (libère la DB).
```

🛟 Récupération si le node custom a été stoppé AVANT l'export : il ne
redémarre plus (il tente d'instancier le runtime @300 au best block → host
function manquante → txpool essentiel meurt). Solution : **omni-node lit la
DB** — le lancer SANS `--collator` (mêmes `--chain`/`--base-path`/relay
args), puis `cutover.mjs export-head ws://127.0.0.1:9988`. Le best block
d'omni-node = H. Penser à régénérer `melodie3-fork-para.json` depuis le
fork courant (le genesis change à chaque re-fork).

---

## Phase 4 — Enregistrement sur la relay locale

Sur paseo-local, sudo = Alice (raccourci LOCAL ; sur le vrai Paseo ce sera
`registrar.reserve` + `registrar.register` + coretime) :

```sh
node scripts/cutover.mjs register ws://127.0.0.1:<RELAY_WS> 2000 genesis-head.hex $WASM
```

Le script fait la séquence COMPLÈTE (vécue en répétition le 2026-06-10) :
1. `sudo(batchAll[configuration.setCoretimeCores(1), paraSudoWrapper.sudoScheduleParaInitialize(2000, {genesisHead, validationCode, paraKind: true})])`
   — ⚠️ un réseau zombienet lancé SANS `[[parachains]]` provisionne **0 core**
   (`schedulerParams.numCores: 0`) : sans le `setCoretimeCores`, la para reste
   muette même enregistrée. NB : le pallet s'appelle `paraSudoWrapper`
   (singulier) sur Paseo.
2. attente de l'activation de la config (frontière de session) ;
3. `sudo(coretime.assignCore(0, 0, [[{Task: 2000}, 57600]], null))` — core 0
   épinglé à plein temps sur la para.

Piège si on passe par `registrar.register` (polkadot-js) à la place : on
obtient un **Parathread sans core** → aucun bloc. Rattrapage :
`sudo(paraSudoWrapper.sudoScheduleParathreadUpgrade(2000))` + les étapes
cores ci-dessus.

Vérifs : `paras.paraLifecycles(2000)` = `Parachain`, `paras.heads(2000)` ==
le header exporté, et `parachainHost.claimQueue()` contient 2000.

✅ Critère : à la frontière de session suivante, `paras.heads(2000)` sur la
relay == header(H).

---

## Phase 5 — Handoff omni-node sur la MÊME DB

```sh
# 5.1 spec collator = spec forké + extensions omni-node (genesis identique !)
jq '. + {relay_chain: "paseo-local", para_id: 2000}' \
  melodie3-fork.json > melodie3-fork-para.json
# garder le même "id" ("melodie3_fork") → même chemin chains/<id>/db

# 5.2 copier le spec relay du réseau zombienet EN COURS vers un chemin stable
#     (omni-node ne sait pas résoudre l'id `paseo-local` tout seul, et le
#     dossier temp zombienet meurt avec le réseau — à refaire à chaque
#     nouveau `pop up`, le genesis relay change !)
cp "$(ls -td /var/folders/*/T/zombie-* /tmp/zombie-* 2>/dev/null | head -1)/paseo-local.json" \
   ./paseo-local-relay.json

# 5.3 lancer omni-node SUR la base-path du node custom ; les args après `--`
#     vont à la partie relay (le spec du réseau qui tourne)
polkadot-omni-node \
  --chain melodie3-fork-para.json \
  --collator --alice \
  --base-path /tmp/rehearsal/solo \
  --database paritydb \
  --relay-chain-rpc-urls ws://127.0.0.1:<RELAY_WS> \
  --rpc-port 9988 \
  -- --chain ./paseo-local-relay.json \
     --network-backend libp2p \
     --bootnodes <multiaddr alice> --bootnodes <multiaddr bob>
```

Les multiaddrs des validateurs sont dans `zombie.json` du dossier zombienet
(`grep -o '"/ip4[^\"]*"' <zombie-dir>/zombie.json`).

⚠️ Troubleshooting vécu (2026-06-10) : symptôme `Collation wasn't advertised
to any validator` + le collateur re-propose en boucle depuis H avec des
reorgs, `finalized` figé à H. Diagnostic : `curl -s 127.0.0.1:9616/metrics |
grep peers_count` → **0 pairs côté relay minimal**. L'advertisement des
collations passe par le P2P (pas par le RPC), les validateurs zombienet
écoutent en WebSocket, et le backend litep2p par défaut ne s'y connectait
pas → `--network-backend libp2p` sur les args relay (+ bootnodes explicites).
Vérifier aussi que le genesis du spec relay copié == `chain_getBlockHash(0)`
de la relay qui tourne (les noms de protocole P2P embarquent ce hash : un
mismatch = rejet silencieux des handshakes).

Avertissement attendu au démarrage : `The parachain system pallet is missing
from the runtime's metadata` — omni-node inspecte le runtime du GENESIS du
spec (= le runtime solo @201, sans cumulus). Cosmétique : la tête de la DB
(bloc H) porte le `:code` @300 qui a bien parachain-system.

✅ Critères, dans l'ordre :
1. omni-node **ouvre la DB existante** sans resync (CHECKPOINT db-handoff —
   le test le plus important de la répétition côté infra) ;
2. il voit head = H via la relay et produit **H+1** ;
3. logs de H+1 : `runtime::solo-to-para` ;
4. H+2, H+3… s'enchaînent à ~6 s.

---

## Phase 6 — Checklist de vérification post-migration

Sur `ws://127.0.0.1:9988` (polkadot-js) :

- [ ] `state_getRuntimeVersion` → `allfeat-melodie-3` / specVersion **300** / txVersion 4
- [ ] `System.LastRuntimeUpgrade` = (300, allfeat-melodie-3)
- [ ] **ATS intacts** : `AtsRegistry` / `AtsVersions` == valeurs pré-fork
      (échantillon noté en Phase 1) ; storage version ATS = 2 (déjà migrée sur
      le live — aucune migration ATS n'est embarquée dans le runtime @300)
- [ ] holds ATS toujours en place sur les comptes déposants
- [ ] MIDDS : préfixes vides, `DepositBase`/`DepositPerByte` re-seedés,
      plus AUCUN hold MusicalWorks/Recordings/Releases (remboursés)
- [ ] `CollatorSelection.Invulnerables` = [Alice], `ParachainInfo` = 2000,
      `Session.NextKeys` au nouveau format `{aura}`
- [ ] **tx d'un compte pré-existant du fork** (nonce > 0) : transfert OK
      → continuité des nonces démontrée
- [ ] créer un NOUVEL ATS → OK ; nouveau dépôt MIDDS → bond au tarif seedé
- [ ] mesurer le poids/PoV réel de H+1 (relay : taille du PoV du candidate)
      → décision finale multi-block ou pas
- [ ] passage d'une rotation de session (6 h en config… raccourcir
      temporairement `Period` pour le test si besoin) : Alice reste collateur

---

## Phase 7 — Industrialisation

- Scripter les phases 1→5 (un script de fork, un script de cutover) pour une
  répétition reproductible en < 30 min.
- Rejouer une 2ᵉ fois de bout en bout sans intervention manuelle.
- Transposer en runbook Paseo réel : `registrar.reserve` → remplacer
  `PARA_ID` + rebuild → coretime (bulk/on-demand) → fenêtre d'annonce →
  setCode avec la **vraie clé sudo** → register → handoff DB sur les machines
  des validateurs.

## Outillage à écrire (dans l'ordre)

1. **`scripts/fork-melodie.{ts,sh}`** — export paged de l'état + patch du
   tableau Phase 1.2 + assemblage du chain-spec forké. (Le plus gros morceau.)
2. **`scripts/cutover.ts`** — setCode (sudoUncheckedWeight) → attente du halt →
   extraction header(H) → sudoScheduleParaInitialize sur la relay.
3. `network-rehearsal.toml` (Phase 2).
