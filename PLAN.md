## 🎯 Goal
Build a fully–featured **Automated Market Maker (AMM)** dApp on Hylé that supports creating liquidity pools and swapping any token pair.  
The implementation **must follow** the exact project structure & patterns already present in this scaffold (contracts ↔ server ↔ front, with indexer + autoprover) and leverage **proof composition** with the existing `hydentity` & `hyllar` crates.

---
## 🗂️ High-Level Work-Breakdown Structure
1. **Project / Workspace Plumbing**  
   1.1  Add two new contract crates under `contracts/`:
   - `token` – ERC-20-like fungible token (generic; mintable by creator).  
   - `amm`   – Core AMM logic (Uniswap-v2 style x*y=k, supporting any two `token` contracts).
   1.2  Register both crates in the workspace `Cargo.toml` and mirror the same build pipeline (`build.rs`, `.img`, `.txt`, client/tx_executor_handler).  
   1.3  Add feature‐gated `client`, `indexer`, and `risc0` support identical to `contract1`.

2. **`token` Contract Implementation**  
   2.1  State: `balances: HashMap<Identity, u128>`, `total_supply`, `allowances` (optional).  
   2.2  Actions enum: `Mint`, `Burn`, `Transfer`, `Approve`, `TransferFrom`.  
   2.3  Implement `ZkContract`:
        • `execute()` matches on action, mutates state, and outputs event string.  
        • `commit()` returns borsh-serialized state hash.  
   2.4  Provide helper `TokenAction::as_blob(contract_name)` like scaffold.  
   2.5  **Proof-composition hooks**: implement the `ERC20` trait from `hyle_hyllar` crate ([source](https://github.com/Hyle-org/hyle/blob/main/crates/contracts/hyllar/src/erc20.rs)) which includes helpful methods like `check_transfer()` and `check_transfer_from()` for composing proofs.

3. **`amm` Contract Implementation**  
   3.1  State: `pools: HashMap<(ContractName, ContractName), PoolState>` where `PoolState { reserve_a, reserve_b, lp_total_supply }`.  
   3.2  Actions enum:
        • `CreatePool { token_a, token_b, amount_a, amount_b }`  
        • `AddLiquidity { token_a, token_b, amount_a_desired, amount_b_desired }`  
        • `RemoveLiquidity { token_a, token_b, lp_amount }`  
        • `Swap { token_in, token_out, amount_in, min_amount_out }`  
   3.3  Business rules: constant-product pricing, 0.3% fee stored in reserves.  
   3.4  In `execute()` for each action:
        • Compose proofs with **both token contracts** (and identity):
          - Use `ERC20::check_transfer()` to validate token transfers using the execution context.
          - Ensure the corresponding `Transfer` operations succeed across both token contracts.
          - Fail if validation mismatches reserves math.  
   3.5  Emit clear string output for indexer logs.  

4. **Contract Metadata & Client Glue**  
   4.1  Add `.img` & `.txt` generation via `cargo risczero build-image` in each crate's `build.rs` (copy from scaffold).  
   4.2  Under `src/client/`, add `tx_executor_handler.rs` mirroring `contract1` so server autoprover can import ELF & PROGRAM_ID constants.  

5. **Indexer Implementations**  
   5.1  For **`token`** and **`amm`**, implement `ContractHandler` (see `contract1/src/indexer.rs`).  
   5.2  Expose REST routes:
        - `/state` → raw JSON state  
        - Additional convenience endpoints:  
          • `/balance/{identity}` (token)  
          • `/pool/{token_a}/{token_b}` (amm)  
          • `/price/{token_in}/{token_out}` (amm)  
   5.3  Register indexers in `server/src/main.rs` via `ContractStateIndexer` modules at startup.

6. **Autoprover Extension**  
   6.1  Duplicate pattern from `ProverModule` in docs:  
        • Maintain local mutable instances of `Token`, `Amm`, plus imported `hyllar` & `hydentity` states.  
   6.2  In `handle_blob`, add routes for `token` & `amm` contract names to dedicated `prove_*` fns.  
   6.3  Each prove fn:
        • Run contract locally to update state.  
        • Spawn async Risc0 prover with proper ELF constant.  
        • Submit `ProofTransaction`.  
   6.4  Respect proof-composition: the prover must generate **all proofs for a BlobTx** (identity + tokenA + tokenB + amm) before settlement.  

7. **Server REST API**  
   7.1  Add higher-level endpoints under `/api/amm`:
        - `POST /create_pool`, `POST /add_liquidity`, `POST /remove_liquidity`, `POST /swap`.  
   7.2  Each handler builds a BlobTx with the correct blob sequence:  
        `identity_blob` → `tokenA_blob` → `tokenB_blob` (if needed) → `amm_blob`.  
   7.3  Use `client-sdk::transaction_builder` helpers to serialize & send BlobTx.  

8. **Frontend (optional for later)**  
   8.1  Minimal UI pages using React/TanStack Router: Pools list, Add liquidity, Swap.  
   8.2  Fetch config & call server endpoints; append `.hydentity` suffix automatically.  

9. **Testing & Tooling**  
   9.1  Add unit tests in each contract crate for core math (`amm::calc_output_amount`).  
   9.2  Use `cargo test`, `cargo mcp_cursor_rust_tools_cargo_check` to ensure compile.  
   9.3  Integration test: spin up local devnet, run server in background, issue a swap via REST, assert reserves update.

10. **Deployment & Registration Script**  
    10.1  Extend `server/init.rs` to register `token` & `amm` contracts on startup (if missing).  
    10.2  Configure initial state: mint large supply of tokens to admin identity, create first pool via bootstrap call.  

---
## 🔑 Architectural & Best-Practice Notes
- Always include an **Identity blob** (`hydentity`) as the first blob; enforce `.hydentity` suffix in server-side identity construction.
- **Proof Composition**: each user-facing action maps to **one BlobTx** that lists multiple blobs (identity + ≥1 token + amm).  Settlement waits for all proofs.
- Reuse scaffold macros & helpers (`module_handle_messages!`, `TxExecutorHandler`).  Don't reinvent message bus wiring.
- Keep contract states small; store only hashes on-chain.  Off-chain indexer reconstructs full state.
- Use hyllar crate for base currency (e.g. `HYR` token) and demonstrate AMM pair `HYR/NEW`.
- Leverage the `ERC20` trait from `hyle_hyllar` crate for token validation during proof composition. The [implementation](https://github.com/Hyle-org/hyle/blob/main/crates/contracts/hyllar/src/erc20.rs) provides `check_transfer()` and `check_transfer_from()` to validate token movements across contracts.
- Prefer pure functions for pricing math so both on-chain commit & off-chain indexer can reuse.

---
## ▶️ Suggested Implementation Order for the AI Agent
1. Duplicate `contracts/contract1` → `contracts/token`; adjust names & IDs.  
2. Implement token logic following the `ERC20` trait from `hyle_hyllar`; implement client + indexer; `cargo check`.
3. Duplicate again → `contracts/amm`; implement AMM logic with token-integration via `ERC20` trait methods.
4. Update workspace `Cargo.toml` + top-level `build.rs` if needed.  
5. Extend server `init.rs`, `app.rs`, `ProverModule`.  
6. Add indexer modules registration.  
7. Implement REST routes.  
8. Run `cargo check` & unit tests with mcp tools.  
9. Manual end-to-end test against devnet.

---
## ⏭️ Next Steps After MVP
- Implement fee growth tracking & withdrawable protocol fees.  
- Add support for flash swaps.  
- Front-end charting of price history via indexer queries.
