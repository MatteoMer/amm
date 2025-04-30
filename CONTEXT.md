# Developing on Hylé: Architecture, Key Concepts, and Workflows

**Introduction to Hylé:**  
Hylé is a next-generation Layer-1 blockchain platform designed for building **provable applications** that combine on-chain security with off-chain execution. Unlike traditional blockchains, Hylé doesn’t run your contract logic on-chain; instead, it verifies zero-knowledge proofs of off-chain computations on a minimalist base layer ([Home | Hyle.eu](https://www.hyle.eu/#:~:text=%2A%20Native%20zero,composing%20private%20and%20public%20inputs)) ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#:~:text=An%20app%E2%80%99s%20transactions%20are%20sequenced,store%20the%20state%20commitment%20onchain)). This approach yields high throughput, fast finality, and low fees by doing only the **minimum on-chain** work needed for trust. Applications built on Hylé enjoy: 

- **Off-chain Execution, On-chain Trust:** Complex application logic runs off-chain in any language or environment you prefer, while a succinct proof of its correct execution is verified on-chain ([Home | Hyle.eu](https://www.hyle.eu/#:~:text=%2A%20Native%20zero,composing%20private%20and%20public%20inputs)). This gives you the flexibility of Web2-like performance with Web3-level trustlessness and privacy ([Home | Hyle.eu](https://www.hyle.eu/#:~:text=%2A%20Native%20zero,composing%20private%20and%20public%20inputs)).  
- **Minimal On-Chain State:** Hylé does not store full contract state on-chain – only proofs of state transitions are stored ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#minimal-onchain-state#:~:text=The%20network%20maintains%20proofs%20of,than%20the%20entire%20onchain%20state)). This drastically reduces on-chain storage needs and eliminates the need for traditional **EVMs or execution VMs** ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#minimal-onchain-state#:~:text=Hyl%C3%A9%20does%20not%20include%20a,Virtual%20Machine)). There is no Solidity or EVM; you write your smart contract logic in general-purpose languages (e.g. Rust) and prove its execution off-chain.  
- **Every App as a Rollup:** Each Hylé application behaves like its own zk-rollup on a shared base chain ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#:~:text=Every%20app%20is%20a%20rollup)). Your app maintains its state off-chain, and Hylé’s base layer sequences your transactions and verifies your state transition proofs. This avoids fragmentation (all apps share one base chain) while allowing each app to evolve independently ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#:~:text=Every%20app%20is%20a%20rollup)).  
- **Integrated Privacy:** Privacy isn’t an afterthought – since execution is off-chain, you can keep inputs and data private by only revealing the proof on-chain ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#:~:text=Privacy%20is%20built)). The on-chain proof is public, but it never exposes the underlying private inputs, enabling built-in privacy for applications.  

Hylé essentially “unchains” your app by only putting what’s **sufficient** on-chain ([Home | Hyle.eu](https://www.hyle.eu/#:~:text=%2A%20Native%20zero,composing%20private%20and%20public%20inputs)). As their motto says, you can enjoy *“onchain security and offchain performance”* ([Home | Hyle.eu](https://www.hyle.eu/#:~:text=We%20must%20do%20better)). This re-imagined model requires a different development approach, which we’ll outline below.

## Project Structure and Architecture (Hylé Wallet Demo Pattern)

Building on Hylé typically involves a **three-part project structure**, as exemplified by the official Hylé Wallet scaffold ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=The%20application%20follows%20a%20client,architecture%20where)):

- **Frontend (Client):** The user interface of the application, e.g. a web or mobile app. The frontend is responsible for handling user interactions and sending high-level operation requests to your backend server ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=The%20application%20follows%20a%20client,architecture%20where)). It does **not** contain sensitive keys or heavy logic. In the Hylé Wallet demo, the frontend (under the `front/` directory) is a web app (built with Bun + Vite) that interacts with the backend via HTTP requests. For example, when a user initiates a token transfer or other action, the frontend will call a backend API endpoint (such as “transfer”) rather than directly forming blockchain transactions. 

- **Backend Server:** A server-side component (often written in Rust for close integration with Hylé’s Rust SDK) that handles all blockchain interactions and proof generation ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=The%20application%20follows%20a%20client,architecture%20where)). The server acts as the **transaction coordinator** for your app:
  - It receives requests from the frontend (e.g. “transfer 20 tokens to X”) and translates them into Hylé transactions. This involves **crafting a Blob transaction**, submitting it to the Hylé network, and then generating the corresponding proofs (explained below) ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=,processed%20through%20the%20Hyl%C3%A9%20network)). 
  - It manages any necessary user authentication/identity proofs (since Hylé doesn’t use traditional keypair wallets, your server might help prove the user’s identity or signature).
  - It uses the Hylé SDK or API to send transactions to the Hylé devnet and to listen for results. In the wallet example, the backend is a Rust service (`server/` crate) that uses Hylé’s Rust client SDK to interact with a local Hylé node and to run the proof-generation logic. By centralizing these tasks, the frontend remains lightweight and the complexity of proof management and transaction formatting is kept on the server side.  

- **Smart Contract Logic (ZK Program in Rust):** The business logic of your application is implemented as **off-chain verifiable programs** (often called “contracts” for analogy, though they don’t execute on-chain). In Hylé, you can write this logic in Rust (or other supported languages like Noir) and compile it to a form that can produce a zero-knowledge proof. For example, the Hylé Wallet’s contract logic lives in the `contracts/` directory as a Rust program (leveraging the [RISC0](https://www.risc0.com) zkVM proving system). This Rust code defines what happens when, say, a token transfer or identity verification is executed – e.g., decrementing one account’s balance and incrementing another’s, or checking a user’s credentials. At runtime, this code is executed off-chain (inside a zkVM) to produce a proof that the state transition was valid. 

  - These contract programs are typically compiled to a form (like a RISC-V ELF image for RISC0) that the backend can run in a prover. The **Hylé SDK** provides an interface to invoke the contract logic and get a `HyleOutput` (the standardized output of a Hylé proof) ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=,The%20smart%20contract%20output)). Before your app can use a contract on Hylé, you **register** it on the network with its verification key or code (so that the chain knows how to verify the proofs) – on devnet this is done via a simple command or API call to register the contract by name ([Hylé vs. vintage blockchains - Hylé Developer Hub](https://docs.hyle.eu/concepts/hyle-vs-vintage-blockchains/#:~:text=An%20app%E2%80%99s%20transactions%20are%20sequenced,store%20the%20state%20commitment%20onchain)) ([Your first smart contract - Hylé Developer Hub](https://docs.hyle.eu/quickstart/your-first-smart-contract/#:~:text=On%20the%20devnet%2C%20register%20your,contract%20by%20running)). 

This separation of concerns (frontend UI, backend coordinator, off-chain contract logic) matches the Hylé architecture where **all operations are processed through the Hylé network via the backend server** ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=The%20application%20follows%20a%20client,architecture%20where)). The frontend never directly interacts with the blockchain; the backend abstracts those details. This pattern improves security (private keys or secrets can be kept on the server, not in the browser) and allows heavy tasks like proof generation to run on a server or cloud machine rather than the user’s device.

**Summary – how these pieces work together in workflow:** a user action on the frontend triggers a backend call; the backend loads the appropriate contract logic (Rust code), prepares the input data and current state, and sends a **Blob transaction** to the Hylé chain (via the Hylé node or indexer API). The backend then runs the contract code in a zk-prover to generate a proof of the intended state change, and submits that proof as a **Proof transaction**. The Hylé network verifies the proof and finalizes the transaction on-chain, updating the on-chain state commitments. The backend can query the Hylé indexer for the updated state or wait for an event, and then the frontend is updated with the result (e.g. new balances, confirmation message). This end-to-end flow is achieved with Hylé’s unique transaction model, detailed next.

## Blob Transactions and Pipelined Proving

One of Hylé’s core innovations is its **two-step transaction model**: a separation of **intent vs. proof** known as **pipelined proving** ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=Hyl%C3%A9%20introduces%20a%20novel%20transaction,optimizing%20for%20scalability%20and%20privacy)). Instead of a single transaction that both *specifies* and *executes* a state change (as on Ethereum), Hylé splits an operation into: 

1. **Blob Transaction (BlobTx)** – a transaction that contains the *intent* or outline of a state change, without a proof. This is essentially a container of data that says “I intend to perform X operation on Y contract with these inputs.” The blob tx is quickly sequenced on-chain (included in a block) but *not yet finalized*. It reserves your place in the global order and timestamp, but the network doesn’t verify its correctness at this point ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=This%20separation%20solves%20all%20three,proofs%20and%20can%20parallelize%20actions)) ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=run%20without%20blocking%20other%20transactions,proofs%20and%20can%20parallelize%20actions)). Think of it as a commitment to a state transition that must be proven valid later.  

2. **Proof Transaction** – a transaction that contains the *zero-knowledge proof* attesting to the correctness of the state transition outlined in a prior blob. The proof tx references the earlier blob (and the contract in question) and includes the actual proof plus the resulting state update information ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=A%20proof%20transaction%20includes%3A)). When the proof is verified by Hylé’s validators, the state transition is **settled** (finalized) on-chain ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=in%20a%20single%20step%2C%20Hyl%C3%A9,step%20process%20called%20pipelined%20proving)).

**Blob Transaction Structure:** A BlobTx typically includes an **identity** and one or more **blobs** (operation entries). Each blob targets a specific contract and carries the binary input data for the contract to process ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=A%20blob%20transaction%20consists%20of%3A)). For example, a blob tx for a token transfer might look like this (in simplified JSON form):

```json
{
  "identity": "bob.hydentity",
  "blobs": [
    {
      "contract_name": "hydentity",
      "data": "<...>" 
      // e.g. encodes an operation: VerifyIdentity { account: "bob.hydentity", nonce: 2 }
    },
    {
      "contract_name": "hyllar",
      "data": "<...>" 
      // e.g. encodes: Transfer { recipient: "alice.hydentity", amount: 20 }
    }
  ]
}
``` 

In this example, the BlobTx contains two blobs: one for an **identity check** (using a contract named “hydentity”) and one for the **token transfer** logic (contract “hyllar”). The `identity` field (`"bob.hydentity"`) is Bob’s account identifier on Hylé ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=%7B%20,)). Hylé doesn’t use private-key based accounts in the same way as Ethereum; instead an *identity contract* like “hydentity” can verify the user (similar to a signature check) using a proof. In practice, the first blob often is an identity/auth blob (to prove the sender is authorized), followed by the actual operation blob.

**How BlobTx is used in practice:** When your backend server receives an operation request, it will use Hylé’s SDK to create a blob transaction structure like above. It will fill in the `contract_name` and `data` for each operation the user wants to perform (often one primary operation plus an auth blob). It then submits this blob transaction to the Hylé network via an API call. At this point, the Hylé validators will include the blob in the next block, assigning it an order and timestamp ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=Image%3A%20A%20graph%20with%20Alice%2C,A%2C%20then%20for%20TX%20B)) ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=This%20separation%20solves%20all%20three,proofs%20and%20can%20parallelize%20actions)). **No heavy computation is done on-chain yet** – the blob’s content is not executed by the chain, only recorded.

*Why have this blob step?* It decouples transaction sequencing from proving. The moment the blob is on-chain, the operation’s order is fixed and known, so we can safely generate a proof against a specific **base state**. This solves issues like conflicting transactions and timing unpredictability that can occur if proving had to happen simultaneously with sequencing ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=change%20to%20be%20settled)) ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=The%20time%20spent%20generating%20proofs,their%20transaction%20will%20be%20sequenced)). In other words, the blob reserves a spot so that the **prover knows exactly which prior state to base the proof on**, avoiding race conditions. It also allows other transactions to sequence in parallel without waiting for your proof – greatly improving throughput. *“The blob transaction immediately reserves a place in execution order, allowing proof generation to run without blocking other transactions”* ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=This%20separation%20solves%20all%20three,proofs%20and%20can%20parallelize%20actions)). The chain can accept many BlobTxs quickly, while proofs can be generated off-chain in parallel.

**Proof Transaction:** After submitting the BlobTx, it’s the backend’s job to produce the zero-knowledge proof that each blob was executed correctly. Using the application’s contract logic (e.g. running the Rust code in a zkVM with the blob’s data and the current state), the backend or an automated prover will generate a proof. Each blob in the blob transaction generally requires a separate proof (unless you intentionally compose them into one proof – see *Proof Composition* below) ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=1.%20Blob,the%20state%20change%20for%20settlement)). The backend then submits a ProofTx for each blob. A proof transaction contains the contract name and the proof data (often including the claimed new state or state delta as output) ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=A%20proof%20transaction%20includes%3A)). For example, a proof tx JSON might look like:

```json
{
  "contract_name": "hydentity",
  "proof": "<...>"
}
```

This proof would correspond to the first blob (index 0) in Bob’s transaction above, proving that *“Bob’s identity contract hydentity was executed with the given data and produced the expected state change”*. The proof data internally contains the old and new state (e.g. nonce from 1 to 2 for Bob’s account) and an index indicating it’s for blob 0 ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=The%20binary%20proof%27s%20output%20includes%3A)) ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=,first%20blob%20in%20the%20transaction)). Similarly, a second proof tx for `hyllar` (index 1) carries the proof that the token transfer was executed (e.g. Bob’s balance went from 100 to 80 and Alice’s from 0 to 20) ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=)).

Hylé will verify each proof using the registered verification key or method for that contract. Once **all required proofs are verified**, the original blob transaction is considered **settled** and final ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=1.%20Blob,the%20state%20change%20for%20settlement)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)). At settlement, the state changes described by the blobs are applied to the official state (Hylé only stores a state commitment on-chain, not the full state). If a proof fails or is not submitted before a timeout, the blob transaction will eventually be marked **rejected** and its effects ignored ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=To%20remove%20this%20risk%2C%20Hyl%C3%A9,enforces%20timeouts%20for%20blob%20transactions)) ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=If%20the%20proof%20isn%27t%20submitted,is%20ignored%20for%20state%20updates)). (Hylé uses timeouts to prevent a blob without proof from hanging forever ([Pipelined proving - Hylé Developer Hub](https://docs.hyle.eu/concepts/pipelined-proving/#:~:text=Even%20with%20pipelined%20proving%2C%20sequenced,can%20slow%20down%20the%20network)).) In our example, if Bob never submits the proof for the transfer, the transfer blob will expire and not affect balances; other users’ transactions can still continue after the timeout.

**Developer’s view:** The BlobTx/ProofTx model means as a developer you must handle an asynchronous two-phase commit: send a blob, then later send the proof. Fortunately, the Hylé toolkit simplifies this. In a typical workflow, your backend will call an API to post the blob and immediately begin proof generation. Once the proof is ready, another API call submits it. The Hylé network ensures atomicity – no one can front-run or alter the intent in between, and final state is updated only when everything checks out. This pipelined approach gives users a smoother experience: the initial transaction can be accepted quickly (within one block) so the user sees progress, and finality comes as soon as the proof is done (which can be in parallel with other operations). 

**In practice with the Hylé Wallet demo:** The server uses the SDK to construct blob transactions and sign them with the user’s identity. It posts them to a Hylé devnet node (which could be the local node at `localhost:4321` or a remote devnet endpoint). Then it runs the contract code to produce a proof. In dev mode, this might be sped up (e.g., `RISC0_DEV_MODE=1` allows using a faster proof mechanism for development). Finally, it sends the proof. All these steps are abstracted in the SDK’s transaction workflow. As a developer, you orchestrate them, but you don’t have to handle the cryptographic details – you focus on writing your contract logic and calling the right SDK functions to send transactions and proofs.

## Proof Composition (Cross-Contract & Multi-Proof Transactions)

Hylé enables advanced use-cases where a single high-level operation may involve **multiple contracts or multiple proofs**. Traditionally, making one contract call another (especially in different proving systems) would require either combining their logic into one mega-proof or verifying one proof inside another (recursive proof verification), which is complex and costly ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=In%20zero,different%20proofs%20introduce%20complexity)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Cross,proof%20generation%20and%20verification%20stages)). Hylé’s solution is **native proof composition**, allowing you to compose proofs **like** you would compose function calls, but without merging them into one circuit ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=We%20solve%20this%20issue%20by,the%20language%20that%20works%20best)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)).

**What is Proof Composition?** It means you can have a blob transaction that contains several blobs (calls to different contracts, possibly using different proof systems), and Hylé will verify all their proofs together *as part of one atomic operation*. Each proof remains independent – generated separately, potentially in different languages or VMs – but the Hylé network treats them as linked to the same BlobTx and only finalizes if **all** proofs succeed ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Since%20proofs%20in%20Hyl%C3%A9%20remain,using%20its%20optimal%20proving%20scheme)). In other words, Hylé allows **batching multiple proof verifications in one transaction** (the settlement of the blob tx). This eliminates the need to do on-chain recursive proofs. As the docs put it: *“Program A can specify: ‘This only applies if all blobs in this operation are valid.’ At settlement, all proofs are verified together, and the entire operation fails if any proof fails.”* ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=composability)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)). 

**How it works:** Suppose your application workflow involves two separate contracts – for example, a Ticketing contract and a Payment contract – and you want to ensure a user either **buys a ticket with payment or the whole operation aborts**. With Hylé, your backend can create a single BlobTx containing two blobs: one for `TicketApp::purchase()` and one for `MoneyApp::transfer()`. Each of these will be proven by its respective zk program (perhaps one using a RISC0 proof, the other using a different proving system). You don’t have to merge these circuits; you generate two proofs. When submitting the proofs, you can either send them one by one or together – Hylé will wait until it has received a valid proof for each blob in that transaction. Once all proofs are in, Hylé will “settle” the blob tx, updating both the ticket state and the payment state simultaneously. If any proof is invalid or missing by the deadline, the whole transaction is rejected (neither tickets nor payment state change) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Proof%20composition%20is%20useful%20if%3A)).

Crucially, each proof can use the **optimal proving scheme** for that contract ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Proofs%20using%20different%20schemes)). For instance, if `TicketApp` is implemented in one language (say Noir) and `MoneyApp` in another (say Rust with RISC0), each can produce a proof in its own format. Hylé’s verifier supports multiple schemes natively and can handle verifying a Noir proof and a RISC0 proof in parallel within the same block ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Proofs%20using%20different%20schemes)). This means you *“don’t have to compromise or unify proving systems”* – you keep the advantages of specialized circuits ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Since%20proofs%20in%20Hyl%C3%A9%20remain,using%20its%20optimal%20proving%20scheme)).

**When to use proof composition:** You’ll leverage this feature whenever an operation spans **multiple contracts or services** and you need an **atomic result** ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=When%20to%20use%20proof%20composition)). Examples:
- **Cross-app calls:** e.g. an app that uses another app’s service – like a game that uses a token from a DeFi app. You can perform an action in the game and a token transfer in one transaction so that either both succeed or both fail together.
- **Multi-language proofs:** e.g. part of your logic is best written in a high-level DSL while another part needs low-level control or a different zkVM. You can keep them separate but still execute them as one user action.
- **Modular contract design:** break your application logic into multiple small zk programs (for modularity or parallel proving) and then combine them at runtime when needed.

If your operation’s logic is entirely within one proof, composition isn’t needed (it adds no overhead or benefit in that case) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=,now%20afford%20to%20use%20them)). But it’s a powerful tool for building **composable dApps**: Hylé lets one proof assert that *“call X in contract A returned result Y”* and another that *“call Z in contract B succeeded”*, and have the chain enforce that both are valid simultaneously ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Writing%20a%20cross)).

**Developer usage:** From a developer’s perspective, using proof composition means crafting a blob transaction with multiple blobs in it (one per contract call involved). Your backend might orchestrate calls between services to gather all needed blob data. For example, your Ticket purchase backend might first call a MoneyApp API to get a blob for the payment, then include that blob in its own blob list for the TicketApp transaction ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Image%3A%20A%20ticket%20purchase%20process,10%20and%20receives%20their%20ticket)). Each contract’s code can be written assuming it runs standalone (no need to code a verifier for the other contract). You simply indicate in each proof that it’s part of a combined operation. Hylé’s runtime links them via the blob transaction. Proof generation can be done in parallel threads or even by different services, and you submit the proofs as they become ready. In fact, Hylé allows proofs in a multi-proof transaction to be submitted asynchronously – *“as soon as one proof is ready, it can be verified on Hylé, even if the other proofs aren't ready yet… Once all proofs are verified, the transaction is settled”* ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=When%20you%20submit%20multiple%20proofs,proof%20generation%20can%20be%20parallelized)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Thanks%20to%20pipelined%20proving%2C%20proof,proof%20generation%20to%20be%20parallelized)). This maximizes throughput: proving times don’t compound, and you don’t have to wait for the slowest proof to start verifying the faster one ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=When%20you%20submit%20multiple%20proofs,proof%20generation%20can%20be%20parallelized)). The end result is a seamless cross-contract workflow that feels like making multiple contract calls in one atomic function. 

In summary, **Proof Composition** in Hylé gives you the *atomicity of a multi-call transaction* (like Ethereum’s multiple contract calls in one transaction) **without requiring a single monolithic proof**. Each proof remains independent and optimized, and Hylé handles the rest ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Hyl%C3%A9%20gets%20rid%20of%20recursion,fails%20if%20any%20proof%20fails)) ([Proof composition - Hylé Developer Hub](https://docs.hyle.eu/concepts/proof-composability/#:~:text=Since%20proofs%20in%20Hyl%C3%A9%20remain,using%20its%20optimal%20proving%20scheme)).

## Autoprover: Automated Proof Generation Service

After understanding blob and proof transactions, you might wonder: who or what actually generates these proofs in real time? In the Hylé model, proof generation happens off-chain, and it can be performed by the app backend or delegated to a service. **Autoprover** refers to an automated service or process that takes on the task of generating proofs for blob transactions, so developers and users don’t have to manually intervene in that second step.

**What the Autoprover does:** An autoprover continuously monitors for new blob transactions that require proofs (for example, blob transactions from your application or specific contracts), and when it detects one, it automatically runs the corresponding contract code in a prover to produce the ZK proof. In a simple setup, your own backend server can serve as the autoprover – as soon as it submits a BlobTx, it invokes the proving code. In more complex setups, you might have a separate dedicated process (or even an external service) watching the network for your app’s blobs and handling proof generation. Either way, the process is “fire-and-forget” from the user’s perspective: once the blob is out, an autoprover will ensure the proof is computed and sent, without the user needing to do anything else.

**When/how it’s triggered:** The autoprover is typically triggered right after a blob transaction is accepted into the sequence (i.e., after it’s in a block or mempool with a known sequence number and base state). In a dev environment, your backend might receive a confirmation that the blob was included (or you already know it will be, since you’re running a local node). At that moment, it has all the info needed (the state and blob data) to generate the proof. It then runs the ZK proving process. Depending on the proving system, this could take milliseconds to seconds (or more, for very complex logic, though Hylé encourages keeping proofs efficient). The autoprover will then automatically submit the resulting proof transaction back to the network via the API. 

In the Hylé wallet example, the server component essentially acts as an autoprover: after sending a blob, it calls into the RISC0 guest code to produce a proof (using `risc0` methods in Rust) and then uses the SDK to send the proof tx. This all happens under the hood when you run `cargo run -p server` with the proper settings ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=In%20your%20Hyl%C3%A9%20repository%3A)) (with `RISC0_DEV_MODE=1`, it can even use a shortcut mode for faster dev proofs). From a workflow perspective, **the developer doesn’t have to manually pause and generate a proof** – the tooling handles it in sequence. 

## Using the Hylé Devnet Indexer and APIs

To build and debug your Hylé application, you will rely on the **Hylé Devnet** and its provided tools such as the **Indexer API** and SDKs. Hylé provides a developer-friendly OpenAPI (Swagger) specification for all its endpoints, so you can programmatically interact with the network without relying on low-level RPC calls. The indexer is a component that **tracks blockchain data (transactions, states, events)** in a queryable database (the devnet runs an integrated PostgreSQL indexer by default ([Run your local devnet - Hylé Developer Hub](https://docs.hyle.eu/quickstart/devnet/#:~:text=For%20a%20single,the%20hyle%20repository%20and%20run))). This makes it easy to retrieve information like the latest state of a contract or the history of an identity, which would otherwise be complicated since the on-chain data is just proofs and commitments.

**Devnet and Swagger UI:** When you run a local Hylé node in devnet mode (using `cargo run -- --pg` as per quickstart ([Run your local devnet - Hylé Developer Hub](https://docs.hyle.eu/quickstart/devnet/#:~:text=For%20a%20single,the%20hyle%20repository%20and%20run))), it starts an API server (by default at `localhost:4321`). You can access an interactive API documentation at `http://localhost:4321/swagger-ui/` that lists all endpoints and their usage ([API - Hylé Developer Hub](https://docs.hyle.eu/tooling/api/#:~:text=To%20explore%20available%20endpoints%20and,understand%20the%20API%20structure)). Hylé also hosts a public Swagger UI for the devnet indexer at ** ([API - Hylé Developer Hub](https://docs.hyle.eu/tooling/api/#:~:text=2.%20Open%20http%3A%2F%2Flocalhost%3A4321%2Fswagger))** (which you can open in a browser). This documentation shows how to format requests for various actions.

**Key API endpoints and usage:** Through the Hylé API (whether local or via the indexer), you can perform tasks such as:
- **Submitting Transactions:** Endpoints exist to submit blob transactions and proof transactions (often a unified `/transactions` endpoint where the payload determines if it’s a blob or proof, or separate endpoints). Using these, your backend can send the BlobTx JSON (as shown above) to the network, and later send the proof. The Swagger will detail the required fields (e.g. contract names, identity, blob data encoding, etc.). This replaces having to use a CLI tool – you can integrate these calls directly in your application code (e.g., an HTTP POST with the transaction data).
- **Contract and State Queries:** You can query the indexer for the state of a given contract or the outcome of the latest proven transaction for that contract. For example, if you have a token contract maintaining balances, the indexer might provide an endpoint to get the current balance for an identity. Under the hood, the indexer knows the state commitments and can reconstruct state by applying all proven state transitions. The exact endpoints might be like `/contract/{name}/state` or part of the Explorer API. The Hylé Explorer (called **Hyleou**) uses these APIs to show human-readable state on the devnet explorer site.
- **Identity and Account Info:** Since identities are also contract-based, there could be endpoints to look up an identity (like “bob.hydentity”) to fetch associated info (nonce, any public data).
- **Block and Transaction Logs:** You can retrieve lists of transactions, check the status of a particular blob transaction (whether it’s still pending proof or has been settled), and see if any failed. The indexer can return the details of a proof (like the output state diffs) so you can update your off-chain state mirrors.
- **Contract Registration and Metadata:** When deploying a new contract, you’ll use an endpoint or command to register it. On devnet, this was done with a CLI command for simplicity ([Your first smart contract - Hylé Developer Hub](https://docs.hyle.eu/quickstart/your-first-smart-contract/#:~:text=On%20the%20devnet%2C%20register%20your,contract%20by%20running)), but the Swagger API also likely has a route (or it can be done by submitting a special transaction) to register a contract’s verification key or program ID. This allows the chain to start accepting proofs for that contract. The indexer can confirm whether a contract is registered and maybe list all registered contracts.

All of these capabilities are documented in the OpenAPI spec. The Swagger UI provides a convenient way to test them out: for instance, you can fill in a blob transaction JSON and execute it directly from the browser to see the response. On the public devnet indexer UI ([API - Hylé Developer Hub](https://docs.hyle.eu/tooling/api/#:~:text=2.%20Open%20http%3A%2F%2Flocalhost%3A4321%2Fswagger)), you’ll see a read-only version (likely it doesn’t allow executing writes for security). For development, you run your own node+indexer for full control.

**Using SDKs vs raw API:** Hylé offers official SDKs in multiple languages to simplify working with these APIs. For example, there is a **JavaScript SDK** (`@hyle/sdk` on npm) and a **Rust SDK** (available on crates.io) ([API - Hylé Developer Hub](https://docs.hyle.eu/tooling/api/#:~:text=We%20currently%20support%20Rust%20and,JS%20environments)). These provide high-level functions and objects so you don’t have to manually craft HTTP requests. In the Hylé Wallet example, the backend Rust server uses the Rust SDK to, say, construct a `Transaction` object, sign it, and send it, and to trigger proof generation with a single call. Similarly, a frontend could use the JS SDK (`hyle-js` library) to interact with the network (though in the wallet demo the frontend mainly talks to the backend, which then uses Rust). If you prefer not to run a backend, you could move more logic client-side using the JS SDK – for instance, a browser app could directly call the Hylé devnet endpoints (since Hylé has no MetaMask-like wallet requirement, you could authenticate via WebAuthn or any identity and then use the SDK to send transactions). In either case, using the SDK ensures you format things correctly for Hylé’s API and handle signatures/proofs properly.

**Devnet Indexer for frontend updates:** Another use of the indexer is to feed real-time updates to your frontend. You might poll the indexer for transaction status or use a WebSocket if provided. For example, after your backend submits a blob tx, your frontend could call a “status” API (through your backend or directly to indexer) to see if the proof is completed. Alternatively, your backend can push an update to the frontend when the proof is done. The indexer’s data (like an event or a transaction receipt) would indicate that the transaction has been verified successfully. Many developers integrate this into their workflow for a smooth UX (e.g., show pending status, then confirmed).

**Referencing the Swagger:** It’s highly recommended to open the Swagger UI while developing. It will list endpoints like `POST /transactions/blob`, `POST /transactions/proof`, `GET /identity/{id}`, etc., with example payloads. The devnet’s Swagger is available here: ** ([API - Hylé Developer Hub](https://docs.hyle.eu/tooling/api/#:~:text=2.%20Open%20http%3A%2F%2Flocalhost%3A4321%2Fswagger))**. This acts as both documentation and a testing tool. Keep in mind that Hylé is evolving, so endpoints or field names might change; always align with the latest Swagger and Hylé Developer Hub docs for updates.

**Putting it together (frontend-backend-indexer workflow):** In the context of the official wallet architecture:
1. **Frontend** – initiates an action via an HTTP call to your backend (e.g., `POST /sendCoins` with form data).
2. **Backend** – uses Hylé SDK to craft a blob transaction (including, say, an identity blob + the action blob). It posts this to the Hylé node (e.g., via the indexer’s `/transactions` endpoint). The backend gets back a response (e.g. a transaction hash or ID).
3. **Backend** – immediately (or upon callback) runs the autoprover. For example, it calls the local function that executes the Rust contract logic with the provided inputs and initial state (fetched via indexer or cached) to generate a proof. Once done, it submits the proof via the API.
4. **Hylé Network** – upon receiving the proof, verifies it. The indexer updates the status of the blob transaction to “settled” and records the new state.
5. **Backend** – becomes aware of the successful settlement (either by polling the indexer or subscribing to an event). It can then query the indexer for updated state (e.g., the new balances after transfer).
6. **Backend** – responds to the frontend (if it was waiting on a long-poll or via a WebSocket message) that the operation succeeded, including any new data the frontend needs (like updated balances).
7. **Frontend** – updates the UI to reflect the final state (transaction confirmed, new token balances shown, etc.).

Throughout this process, the **Hylé devnet indexer API** is the glue for retrieving information and sending transactions. By following this pattern and utilizing the provided SDKs, developers can implement the full workflow without ever using a CLI manually – everything can be done in code and through HTTP calls.

## Conclusion

In summary, developing on Hylé involves a new mental model where your dApp is split between off-chain logic and on-chain verification. You structure your project with a clear separation of frontend, backend, and provable contract code. You leverage Hylé’s blob/proof transaction pipeline to achieve both performance and security: **Blob transactions** let you enqueue intents instantly, and **Proof transactions** finalize them trustlessly. You can compose multiple proofs across contracts using Hylé’s **proof composition** to build rich, interoperable functionality. The heavy lifting of proof generation can be handled by an **autoprover** component, ensuring a seamless user experience. And with the **Hylé devnet indexer and APIs**, you have the tools to deploy, monitor, and interact with your app’s on-chain footprint with ease – all accessible via standard web requests and documented on Swagger ([API - Hylé Developer Hub](https://docs.hyle.eu/tooling/api/#:~:text=2.%20Open%20http%3A%2F%2Flocalhost%3A4321%2Fswagger)).

With this understanding of Hylé’s architecture and workflows, a blockchain-savvy developer can start building “minimally on-chain, maximally off-chain” applications. The official Hylé Wallet demo provides a working template ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=The%20application%20follows%20a%20client,architecture%20where)) to follow. By using the described project structure and tools, you can focus on writing your application’s logic in Rust (or your preferred language) and trust Hylé’s base layer to secure and settle your app’s state transitions. Happy hacking on Hylé! ([GitHub - Hyle-org/wallet](https://github.com/Hyle-org/wallet#:~:text=The%20application%20follows%20a%20client,architecture%20where)) ([Transactions on Hylé - Hylé Developer Hub](https://docs.hyle.eu/concepts/transaction/#:~:text=Hyl%C3%A9%20introduces%20a%20novel%20transaction,optimizing%20for%20scalability%20and%20privacy)) 


You are an AI assistant tasked with understanding and extending a Hylé wallet codebase. Focus on the **server** and **contract** components. Analyze the structure, the key Hylé-specific libraries, and how they interconnect. Use the examples below as a scaffold to generate new code or documentation that follows the same patterns.

---

# App structure
## Server Component

### 1. Main Entry (`main.rs`)
- **Purpose**: Loads configuration, sets up tracing/logging, initializes Hylé modules (App, Indexer, Prover, DA Listener, REST API), and starts the runtime.  
- **Key Hylé imports**:
  ```rust
  use hyle::utils::conf::Conf;
  use hyle::utils::logger::setup_tracing;
  use hyle::bus::{SharedMessageBus, metrics::BusMetrics};
  use hyle::utils::modules::ModulesHandler;
  use hyle::model::CommonRunContext;
  use hyle::indexer::{
      contract_state_indexer::{ContractStateIndexer, ContractStateIndexerCtx},
      da_listener::{DAListener, DAListenerCtx},
  };
  use hyle::rest::{RestApi, RestApiRunContext};
  ```  

- **Example: loading config & tracing**  
  ```rust
  let config = Conf::new(args.config_file, None, Some(true))
      .context("reading config file")?;
  setup_tracing(&config, format!("{}(nopkey)", config.id.clone()))
      .context("setting up tracing")?;
  ```  

- **Example: creating the message bus and modules handler**  
  ```rust
  let bus = SharedMessageBus::new(BusMetrics::global(config.id.clone()));
  let mut handler = ModulesHandler::new(&bus).await;
  ```  

---

### 2. Contract Initialization (`init.rs`)
- **Purpose**: Registers or validates on-chain contracts via the node and indexer clients, ensuring the correct program ID and initial state.  
- **Key Hylé imports**:
  ```rust
  use client_sdk::rest_client::{NodeApiHttpClient, IndexerApiHttpClient};
  use sdk::{api::APIRegisterContract, ContractName, ProgramId, StateCommitment};
  ```  

- **Example: defining a contract to initialize**  
  ```rust
  pub struct ContractInit {
      pub name: ContractName,
      pub program_id: [u8; 32],
      pub initial_state: StateCommitment,
  }
  ```  

- **Example: registering a contract if missing**  
  ```rust
  node.register_contract(&APIRegisterContract {
      verifier: "risc0-1".into(),
      program_id: ProgramId(contract.program_id.to_vec()),
      state_commitment: contract.initial_state,
      contract_name: contract.name.clone(),
  }).await?;
  wait_contract_state(indexer, &contract.name).await?;
  ```  

---

### 3. Application Module (`app.rs`)
- **Purpose**: Defines the REST endpoints, CORS middleware, and message‐bus integration for wallet actions.  
- **Key Hylé imports**:
  ```rust
  use hyle::rest::AppError;
  use hyle::module_handle_messages;
  use hyle::bus::SharedMessageBus;
  use hyle::model::CommonRunContext;
  ```  

- **Example: building the module and merging routes**  
  ```rust
  let api = Router::new()
      .route("/_health", get(health))
      .route("/api/config", get(get_config))
      .with_state(state)
      .layer(cors);
  handler.build_module::<AppModule>(app_ctx.clone()).await?;
  ```  
  citeturn5view0

- **Example: handling bus messages**  
  ```rust
  module_handle_messages! {
      on_bus self.bus,
  };
  ```  

---

## Contract Component

### 1. Core Logic (`lib.rs`)
- **Purpose**: Implements the wallet contract’s state transitions and actions using Hylé’s `sdk::ZkContract` interface.  
- **Key Hylé imports**:
  ```rust
  use sdk::ZkContract;
  use sdk::utils::parse_raw_calldata;
  use sdk::StateCommitment;
  use sdk::RunResult;
  ```  

- **Example: contract entry point**  
  ```rust
  impl sdk::ZkContract for Wallet {
      fn execute(&mut self, calldata: &sdk::Calldata) -> RunResult {
          let (action, ctx) = sdk::utils::parse_raw_calldata::<WalletAction>(calldata)?;
          // … handle RegisterIdentity and VerifyIdentity …
          Ok((res, ctx, vec![]))
      }
      fn commit(&self) -> StateCommitment {
          StateCommitment(borsh::to_vec(&self).unwrap())
      }
  }
  ```  

- **Example: helper to build identity blob**  
  ```rust
  pub fn build_identity_id(account: &str, password: &str) -> Vec<u8> {
      let mut hasher = Sha256::new();
      hasher.update(password.as_bytes());
      // … compute final hash …
      hash_bytes.to_vec()
  }
  ```  

---

### 2. Metadata Constants (`metadata.rs`)
- **Purpose**: Exposes the compiled contract ELF binary and its 32-byte program ID to the server.  
- **Key Hylé imports**:
  ```rust
  pub const WALLET_ELF: &[u8] = wallet::client::metadata::WALLET_ELF;
  pub const WALLET_ID: [u8; 32] = wallet::client::metadata::PROGRAM_ID;
  ```   

---

# Implementing Indexer for Hylé Contracts

To enable state indexing for Hylé contracts, implement the `ContractHandler` trait. This pattern creates a bridge between on-chain contract execution and off-chain state indexing.

## Key Components

```rust
// 1. Required imports for contract indexing
use anyhow::{anyhow, Context, Result};
use client_sdk::{
    contract_indexer::{
        axum::{extract::State, extract::Path, http::StatusCode, response::IntoResponse, Json, Router},
        utoipa::openapi::OpenApi,
        utoipa_axum::{router::OpenApiRouter, routes},
        AppError, ContractHandler, ContractHandlerStore,
    },
    transaction_builder::TxExecutorHandler,
};
use sdk::Hashed;

// 2. ContractHandler implementation
impl ContractHandler for YourContract {
    // Define API endpoints for the contract
    async fn api(store: ContractHandlerStore<YourContract>) -> (Router<()>, OpenApi) {
        // Create routes using OpenApiRouter
        let (router, api) = OpenApiRouter::default()
            .routes(routes!(get_state))
            .split_for_parts();

        // Optionally add additional routes manually
        let router = router.route("/additional/{param}", axum::routing::get(additional_handler));

        (router.with_state(store), api)
    }

    // Handle on-chain transactions and update contract state
    fn handle_transaction(
        &mut self,
        tx: &sdk::BlobTransaction,
        index: sdk::BlobIndex,
        tx_context: sdk::TxContext,
    ) -> Result<()> {
        // Extract blob information
        let sdk::Blob { contract_name, data: _ } = tx.blobs.get(index.0)
            .context("Failed to get blob")?;

        // Create calldata for contract execution
        let calldata = sdk::Calldata {
            identity: tx.identity.clone(),
            index,
            blobs: tx.blobs.clone().into(),
            tx_blob_count: tx.blobs.len(),
            tx_hash: tx.hashed(),
            tx_ctx: Some(tx_context),
            private_input: vec![],
        };

        // Execute contract and process output
        let hyle_output = self.handle(&calldata).map_err(|e| anyhow::anyhow!(e))?;
        let program_outputs = str::from_utf8(&hyle_output.program_outputs).unwrap_or("no output");

        sdk::info!("🚀 Executed {contract_name}: {}", program_outputs);
        Ok(())
    }
}

// 3. Define API endpoint handlers
#[utoipa::path(
    get,
    path = "/state",
    tag = "YourContract",
    responses(
        (status = 200, description = "Get contract state")
    )
)]
pub async fn get_state<S: Serialize + Clone + 'static>(
    State(state): State<ContractHandlerStore<S>>,
) -> Result<impl IntoResponse, AppError> {
    let store = state.read().await;
    store.state.clone().map(Json).ok_or(AppError(
        StatusCode::NOT_FOUND,
        anyhow!("No state found for contract '{}'", store.contract_name),
    ))
}
```

## Usage in Hylé Application

In your main application, register the contract with `ContractStateIndexer`:

```rust
// Register with ContractStateIndexer in your app startup
handler
    .build_module::<ContractStateIndexer<YourContract>>(ContractStateIndexerCtx {
        contract_name: contract_name.into(),
        common: ctx.clone(),
    })
    .await?;
```

## Key Integration Points

1. **API Endpoints**: Define handlers for querying contract state via HTTP endpoints
2. **Transaction Processing**: Handle BlobTransaction processing in `handle_transaction`
3. **State Indexing**: Updates contract state when on-chain transactions are processed
4. **Path Parameters**: Use curly braces for route parameters: `/endpoint/{param}` (not `:param`)
5. **OpenAPI Integration**: Define endpoint documentation with `utoipa::path` attributes


# Implementing Auto Prover for Hylé Contracts

The auto prover is a critical component that automatically generates zero-knowledge proofs for blob transactions in the Hylé architecture. Below is a concise guide to implementing auto proving for contracts.

## Key Components

```rust
// 1. Required imports for auto proving
use anyhow::{anyhow, Result};
use client_sdk::helpers::risc0::Risc0Prover;
use hyle::{
    bus::BusClientSender,
    log_error, module_handle_messages,
    node_state::module::NodeStateEvent,
    utils::modules::{module_bus_client, Module},
};
use sdk::{
    BlobIndex, BlobTransaction, Block, Calldata, Hashed, ProofTransaction,
    TransactionData, TxHash, ZkContract, HYLE_TESTNET_CHAIN_ID,
};

// 2. ProverModule structure holding contract states
pub struct ProverModule {
    bus: ProverModuleBusClient,
    ctx: Arc<ProverModuleCtx>,
    unsettled_txs: Vec<BlobTransaction>,
    contract1: Contract1,
    contract2: Contract2,
    wallet: Wallet,  // Each contract needs a state instance
}

// 3. Bus message handler pattern
module_bus_client! {
    #[derive(Debug)]
    pub struct ProverModuleBusClient {
        sender(AppEvent),
        receiver(NodeStateEvent),
    }
}

// 4. Module implementation
impl Module for ProverModule {
    type Context = Arc<ProverModuleCtx>;

    async fn build(ctx: Self::Context) -> Result<Self> {
        let bus = ProverModuleBusClient::new_from_bus(ctx.app.common.bus.new_handle()).await;
        
        // Initialize contract states
        let contract1 = Contract1::default();
        let contract2 = Contract2::default();
        let wallet = Wallet::default();

        Ok(ProverModule {
            bus,
            ctx,
            unsettled_txs: vec![],
            contract1,
            contract2,
            wallet,
        })
    }

    async fn run(&mut self) -> Result<()> {
        module_handle_messages! {
            on_bus self.bus,
            listen<NodeStateEvent> event => {
                _ = log_error!(self.handle_node_state_event(event).await, "handle node state event")
            }
        };
        Ok(())
    }
}

// 5. Transaction processing and proof generation
impl ProverModule {
    // Process new blocks
    async fn handle_processed_block(&mut self, block: Block) -> Result<()> {
        // Extract blob transactions from the block
        for (_, tx) in block.txs {
            if let TransactionData::Blob(tx) = tx.transaction_data {
                let tx_ctx = sdk::TxContext {
                    block_height: block.block_height,
                    block_hash: block.hash.clone(),
                    timestamp: block.block_timestamp.clone(),
                    lane_id: block.lane_ids.get(&tx.hashed()).unwrap().clone(),
                    chain_id: HYLE_TESTNET_CHAIN_ID,
                };
                self.handle_blob(tx, tx_ctx);
            }
        }
        // Process settled transactions
        for s_tx in block.successful_txs { self.settle_tx(s_tx)?; }
        Ok(())
    }

    // Router for blob transactions
    fn handle_blob(&mut self, tx: BlobTransaction, tx_ctx: sdk::TxContext) {
        for (index, blob) in tx.blobs.iter().enumerate() {
            if blob.contract_name == self.ctx.app.contract1_cn {
                self.prove_contract1_blob(&index.into(), &tx, &tx_ctx);
            }
            if blob.contract_name == self.ctx.app.contract2_cn {
                self.prove_contract2_blob(&index.into(), &tx, &tx_ctx);
            }
            if blob.contract_name == self.ctx.app.wallet_cn {
                self.prove_wallet_blob(&index.into(), &tx, &tx_ctx);
            }
        }
        self.unsettled_txs.push(tx);
    }

    // Contract-specific proving method
    fn prove_wallet_blob(
        &mut self,
        blob_index: &BlobIndex,
        tx: &BlobTransaction,
        tx_ctx: &sdk::TxContext,
    ) {
        // Skip proving for blocks before our start height
        if tx_ctx.block_height.0 < self.ctx.start_height.0 {
            return;
        }
        
        let blob = tx.blobs.get(blob_index.0).unwrap();
        let blobs = tx.blobs.clone();
        let tx_hash = tx.hashed();

        // Initialize the RISC0 prover with contract ELF
        let prover = Risc0Prover::new(wallet::client::tx_executor_handler::metadata::WALLET_ELF);
        
        // Serialize current state for proof generation
        let Ok(state) = self.wallet.as_bytes() else {
            error!("Failed to serialize state on tx: {}", tx_hash);
            return;
        };

        // Create calldata for the prover
        let calldata = Calldata {
            identity: tx.identity.clone(),
            tx_hash: tx_hash.clone(),
            private_input: vec![],
            blobs: blobs.clone().into(),
            index: *blob_index,
            tx_ctx: Some(tx_ctx.clone()),
            tx_blob_count: blobs.len(),
        };

        // Execute contract locally to update state
        if let Err(e) = self.wallet.execute(&calldata).map_err(|e| anyhow!(e)) {
            error!("error while executing contract: {e}");
            self.bus
                .send(AppEvent::FailedTx(tx_hash.clone(), e.to_string()))
                .unwrap();
        }

        // Notify that transaction is sequenced
        self.bus
            .send(AppEvent::SequencedTx(tx_hash.clone()))
            .unwrap();

        // Generate proof asynchronously
        let node_client = self.ctx.app.node_client.clone();
        let blob = blob.clone();
        tokio::task::spawn(async move {
            match prover.prove(state, calldata).await {
                Ok(proof) => {
                    info!("Proof generated for tx: {}", tx_hash);
                    // Submit proof transaction
                    let tx = ProofTransaction {
                        contract_name: blob.contract_name.clone(),
                        proof,
                    };
                    let _ = log_error!(
                        node_client.send_tx_proof(&tx).await,
                        "failed to send proof to node"
                    );
                }
                Err(e) => {
                    error!("Error proving tx: {:?}", e);
                }
            };
        });
    }
}
```

## Usage in Hylé Application

In your main application, register the ProverModule:

```rust
// Initialize prover module in app startup
let prover_ctx = Arc::new(ProverModuleCtx {
    app: app_ctx.clone(),
    start_height,
});

handler
    .build_module::<ProverModule>(prover_ctx.clone())
    .await?;
```

## Key Integration Points

1. **Contract Registration**: Include your contract in the ProverModule struct and initialization
2. **Blob Routing**: Add a condition in `handle_blob` to route blobs to your contract's proving method
3. **Proof Generation**: Implement a contract-specific proving method (`prove_your_contract_blob`)
4. **State Management**: Execute contract logic within the prover to update local state
5. **Proof Submission**: Use RISC0 (or your chosen prover) to generate proofs and submit as ProofTransactions

This pattern enables automatic off-chain proof generation for on-chain blob transactions, completing the Hylé pipelined proving architecture where state transitions are first sequenced (BlobTx) and then proven valid (ProofTx).


This pattern enables your contract to maintain state consistency between the chain and indexer, allowing external services to query contract state through standard HTTP APIs.

# Important notes
- This app is meant for prod and you should not take any shortcut or not implementing something. always go all-in.
- In the way identity works, it should always finish by `.{id_contract_name}`, for example `matteo.hydentity`. it does not change the contract but should be taken into account inside the frontend (manually add it)
- Always implement indexer and autoprover
- Always use proof compositions; usually a blobtx will at least consist of two blobs: identity blob and a contract blob. but for example interacting with a token in a contract will need at least 3: id, token, custom contract.
- hyllar is the default token, hydentity the default identity; you can add them using hyle_hyllar and hyle_hydentity crates