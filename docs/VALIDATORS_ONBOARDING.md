# Validator Node Operator Guide

**Network:** Allfeat
**Consensus:** Proof of Authority (PoA)
**Version:** 1.6

---

## 📝 Prerequisites & Polkadot Reference

The Allfeat blockchain is built using the **Polkadot SDK** (Substrate). Consequently, our node architecture, security best practices, and maintenance procedures are functionally identical to running a Polkadot node.

We **strongly recommend** that operators consult the official **[Polkadot Validator Guide](https://docs.polkadot.com/infrastructure/running-a-validator)**. It provides extensive details on hardware selection, Linux optimization, and advanced security patterns that go beyond the scope of this document.

> **⚠️ CRITICAL DISTINCTION**
> While the _infrastructure_ is similar, the **consensus configuration** differs significantly.
>
> - **Polkadot** uses NPoS (Staking) and BABE.
> - **Allfeat** uses **PoA (Proof of Authority)** and **Aura**.
>
> **How to use these guides:** Use the Polkadot documentation for deep system administration and security hardening, but **strictly follow the specific instructions in this guide** for chain configuration, session keys, and the onboarding process.

---

## 1. Introduction

Welcome to the **Allfeat** network. Unlike public NPoS networks, our blockchain operates on a **Proof of Authority (PoA)** consensus model.

### Key Distinctions

- **Permissioned Entry:** Validators are not selected based on staked tokens. Admission to the active set is granted solely by the Governance Council based on reputation and technical capability.
- **No Staking:** You do not need to bond funds to validate blocks.
- **Session Keys:** We strictly utilize **Aura** (Block Production) and **Grandpa** (Finality).

---

## 2. Validator Selection Policy & Eligibility

Joining the Allfeat active validator set is a privileged role. To ensure the stability and mission-alignment of the network, we adhere to a strict selection policy.

### 🎯 Priority: Music Industry Stakeholders

Our primary goal is to decentralize the network among the actors who actually build the future of music. We prioritize applications from **Music Industry Entities** (Labels, DSPs, Rights Organizations, Tech Providers) who demonstrate the technical capacity to maintain a secure node.

- _Goal:_ Giving governance and validation power back to the industry.
- _Requirement:_ You must prove you have the internal IT resources or an external DevOps partner capable of meeting our SLA.

### 🤝 Strategic Partners: Trusted Web3 & Web2

Especially during the **Mainnet Launch** phase, network stability is non-negotiable. We welcome applications from established **Web3 Infrastructure Providers** and trusted **Web2 Technical Partners**.

- Your role is to provide the bedrock of stability and security while the ecosystem matures.
- We value partners with whom we have established trust and who have a proven track record in managing high-availability infrastructure.

### 🛑 The 12-Node Limit

To maintain consensus efficiency during this initial phase, the active validator set is strictly capped at **12 Nodes**.

- Selection is competitive.
- If the set is full, qualified candidates may be placed on a waitlist or encouraged to run a **Standby Node** (Full Node) to build reputation for future expansion rounds.

---

## 3. Infrastructure Requirements

To ensure consistent block times and network stability, your infrastructure must meet the following benchmarks.

| Component   | Validator Requirement  | Bootnode Requirement (Optional) |
| :---------- | :--------------------- | :------------------------------ |
| **Storage** | **NVMe SSD** (500GB+)  | Standard SSD (200GB+)           |
| **CPU**     | High Single-Core Speed | Mid-range Server CPU            |
| **RAM**     | 16GB - 32GB ECC        | 8GB - 16GB                      |
| **Network** | 1 Gbps Symmetric       | 1 Gbps (High Availability)      |

**Operating System:** Linux (Ubuntu 22.04 LTS or Debian 12).

---

## 4. Network & Security Architecture

**NEVER expose your Validator's RPC ports to the public internet.**

### Firewall Rules (Validator)

- **Allow:** Port `30333` (P2P Traffic) - TCP/UDP.
- **Block:** Port `9933` (HTTP RPC) and `9944` (WS RPC). These must be accessible strictly via `localhost` or a secure VPN tunnel.

### Sentry Node Pattern (Highly Recommended for Validators)

For optimal security, your Validator node should sit in a private subnet, connected only to public-facing "Sentry Nodes" (Full Nodes) that you control. This masks your Validator's real IP address.

---

## 5. Optional Contribution: Public Bootnode

If you wish to further support the ecosystem, you may voluntarily operate a **Public Bootnode**. A bootnode acts as a permanent discovery point for new nodes joining the network; it does not produce blocks but helps with peer discovery.

### Configuration

For detailed instructions on how to generate a stable node key and configure your server as a bootnode, please refer to the official Polkadot documentation:

👉 **[Polkadot Docs: Setup a Bootnode](https://docs.polkadot.com/infrastructure/running-a-node/setup-bootnode/)**

### Submission

Once your bootnode is running, please construct your **Multiaddr** (containing your public IP and persistent Peer ID) and submit it to the administrators. We will add it to the chain specification or public registry.

_Format:_ `/ip4/<YOUR_PUBLIC_IP>/tcp/30333/p2p/<YOUR_PEER_ID>`

---

## 6. Setup & Key Generation (Validator Only)

### Step 1: Install and Sync

Set up your node using the official binary. Configure it as a `systemd` service to ensure it restarts automatically. Ensure NTP (Time Sync) is active.

### Step 2: Generate Session Keys (Local)

We use **Aura** and **Grandpa**. You must generate these keys locally on your server's keystore.

Run the following command on your node (localhost only):

    curl -H "Content-Type: application/json" \
        -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' \
        http://localhost:9933

**Response Example:**

    {
      "jsonrpc": "2.0",
      "result": "0x1234...abcd",
      "id": 1
    }

1.  **Copy the `result` hex string.** This is the concatenation of your public keys.
2.  **Backup your Keystore:** Locate the `keystore` folder in your chain's base path. **If you lose these keys, you cannot validate.**

---

## 7. On-Chain Association

The network requires you to map your local session keys (from Step 2) to an on-chain account.

1.  **Create an Account:** Use the Polkadot.js extension to generate a standard account. This will be your **Validator ID**.
2.  **Navigate to Polkadot.js Apps:** Connect to the network.
3.  **Go to:** `Developer` -> `Extrinsics`.
4.  **Configure the Call:**
    - **Account:** Select your Validator ID account.
    - **Pallet:** Select `session`.
    - **Method:** Select `setKeys`.
    - **keys:** Paste the long hex string (`0x...`) generated in Step 2.
    - **proof:** Enter `0x00`.
5.  **Submit:** Sign and submit the transaction.

---

## 8. Governance Application & Activation

Once your keys are set on-chain, you must request entry into the active set.

### How to Apply

Send a secure communication to the Network Administrators containing:

1.  **Entity Name** (Your organization).
2.  **Validator ID Address** (The account used in the `session.setKeys` transaction).
3.  **(Optional) Bootnode Multiaddr:** If you are running a bootnode (from Section 5).

### Activation Process

1.  **Verification:** Admins will verify your node's health.
2.  **Sudo Execution:** The Governance Council will execute a privileged transaction to add your Validator ID to the Authority Set.
3.  **Epoch Change:** You will become active at the start of the next session. Monitor your logs for: `Prepared block for proposing`.

---

## 9. Maintenance & SLA

- **Uptime:** Maintain >99.9% uptime.
- **Updates:** Apply critical security updates within **24 hours** of release.
- **Monitoring:** Implement Prometheus/Grafana monitoring.

---

## 10. Support & Contact

If this document does not answer all your questions regarding the validator setup:

- **Technical Issues:** Please consider creating an issue on our repository: **[Allfeat/Allfeat](https://github.com/Allfeat/Allfeat)**.
- **Direct Contact:** Email us at **[hello@allfeat.org](mailto:hello@allfeat.org)**.
- **Community:** Join our **Discord server** for real-time support.
