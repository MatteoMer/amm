use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sdk::{Identity, RunResult};

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod indexer;

impl sdk::ZkContract for Token {
    /// Entry point of the contract's logic
    fn execute(&mut self, calldata: &sdk::Calldata) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_calldata::<TokenAction>(calldata)?;

        // Execute the given action
        let res = match action {
            TokenAction::Mint { to, amount } => self.mint(&to, amount)?,
            TokenAction::Burn { from, amount } => self.burn(&from, amount)?,
            TokenAction::Transfer { to, amount } => self.transfer(&calldata.identity, &to, amount)?,
            TokenAction::Approve { spender, amount } => self.approve(&calldata.identity, &spender, amount)?,
            TokenAction::TransferFrom { from, to, amount } => self.transfer_from(&calldata.identity, &from, &to, amount)?,
        };

        Ok((res, ctx, vec![]))
    }

    /// Serialize the full state on-chain.
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(self.as_bytes().expect("Failed to encode Token state"))
    }
}

impl Token {
    pub fn mint(&mut self, to: &Identity, amount: u128) -> Result<String, String> {
        // Add amount to the recipient's balance
        let to_balance = self.balances.entry(to.clone()).or_insert(0);
        *to_balance += amount;
        
        // Increase total supply
        self.total_supply += amount;
        
        Ok(format!("Successfully minted {} tokens to {}", amount, to.0))
    }
    
    pub fn burn(&mut self, from: &Identity, amount: u128) -> Result<String, String> {
        // Check if the account has enough balance
        let from_balance = self.balances.get(from).cloned().unwrap_or(0);
        if from_balance < amount {
            return Err(format!("Insufficient balance: {} < {}", from_balance, amount));
        }
        
        // Reduce the balance
        let new_balance = from_balance - amount;
        if new_balance == 0 {
            self.balances.remove(from);
        } else {
            self.balances.insert(from.clone(), new_balance);
        }
        
        // Decrease total supply
        self.total_supply -= amount;
        
        Ok(format!("Successfully burned {} tokens from {}", amount, from.0))
    }
    
    pub fn transfer(&mut self, from: &Identity, to: &Identity, amount: u128) -> Result<String, String> {
        // Check if sender has enough balance
        let from_balance = self.balances.get(from).cloned().unwrap_or(0);
        if from_balance < amount {
            return Err(format!("Insufficient balance: {} < {}", from_balance, amount));
        }
        
        // Reduce sender's balance
        let new_from_balance = from_balance - amount;
        if new_from_balance == 0 {
            self.balances.remove(from);
        } else {
            self.balances.insert(from.clone(), new_from_balance);
        }
        
        // Increase recipient's balance
        let to_balance = self.balances.entry(to.clone()).or_insert(0);
        *to_balance += amount;
        
        Ok(format!("Successfully transferred {} tokens from {} to {}", amount, from.0, to.0))
    }
    
    pub fn approve(&mut self, owner: &Identity, spender: &Identity, amount: u128) -> Result<String, String> {
        let key = (owner.clone(), spender.clone());
        self.allowances.insert(key, amount);
        
        Ok(format!("Successfully approved {} tokens for {} to spend on behalf of {}", amount, spender.0, owner.0))
    }
    
    pub fn transfer_from(&mut self, spender: &Identity, from: &Identity, to: &Identity, amount: u128) -> Result<String, String> {
        // Check allowance
        let key = (from.clone(), spender.clone());
        let allowance = self.allowances.get(&key).cloned().unwrap_or(0);
        if allowance < amount {
            return Err(format!("Insufficient allowance: {} < {}", allowance, amount));
        }
        
        // Check if sender has enough balance
        let from_balance = self.balances.get(from).cloned().unwrap_or(0);
        if from_balance < amount {
            return Err(format!("Insufficient balance: {} < {}", from_balance, amount));
        }
        
        // Reduce allowance
        let new_allowance = allowance - amount;
        if new_allowance == 0 {
            self.allowances.remove(&key);
        } else {
            self.allowances.insert(key, new_allowance);
        }
        
        // Reduce sender's balance
        let new_from_balance = from_balance - amount;
        if new_from_balance == 0 {
            self.balances.remove(from);
        } else {
            self.balances.insert(from.clone(), new_from_balance);
        }
        
        // Increase recipient's balance
        let to_balance = self.balances.entry(to.clone()).or_insert(0);
        *to_balance += amount;
        
        Ok(format!("Successfully transferred {} tokens from {} to {} on behalf of {}", amount, from.0, to.0, spender.0))
    }
}

// Implement the ERC20 trait from hyle-hyllar for proof composition
impl hyle_hyllar::erc20::ERC20 for Token {
    fn total_supply(&self) -> Result<u128, String> {
        Ok(self.total_supply)
    }

    fn balance_of(&self, account: &str) -> Result<u128, String> {
        let identity = Identity(account.to_string());
        Ok(self.balances.get(&identity).cloned().unwrap_or(0))
    }

    fn transfer(&mut self, sender: &str, recipient: &str, amount: u128) -> Result<(), String> {
        let from = Identity(sender.to_string());
        let to = Identity(recipient.to_string());
        
        self.transfer(&from, &to, amount).map(|_| ())
    }

    fn transfer_from(&mut self, spender: &str, owner: &str, recipient: &str, amount: u128) -> Result<(), String> {
        let spender = Identity(spender.to_string());
        let from = Identity(owner.to_string());
        let to = Identity(recipient.to_string());
        
        self.transfer_from(&spender, &from, &to, amount).map(|_| ())
    }

    fn approve(&mut self, owner: &str, spender: &str, amount: u128) -> Result<(), String> {
        let owner = Identity(owner.to_string());
        let spender = Identity(spender.to_string());
        
        self.approve(&owner, &spender, amount).map(|_| ())
    }

    fn allowance(&self, owner: &str, spender: &str) -> Result<u128, String> {
        let owner = Identity(owner.to_string());
        let spender = Identity(spender.to_string());
        let key = (owner, spender);
        
        Ok(self.allowances.get(&key).cloned().unwrap_or(0))
    }
    
    fn check_transfer(
        mut exec_ctx: sdk::caller::ExecutionContext,
        recipient: &str,
        amount: u128,
    ) -> Result<(), String> {
        exec_ctx.is_in_callee_blobs(
            &exec_ctx.contract_name.clone(),
            hyle_hyllar::HyllarAction::Transfer {
                recipient: recipient.to_string(),
                amount,
            },
        )
    }

    fn check_transfer_from(
        mut exec_ctx: sdk::caller::ExecutionContext,
        owner: &str,
        recipient: &str,
        amount: u128,
    ) -> Result<(), String> {
        exec_ctx.is_in_callee_blobs(
            &exec_ctx.contract_name.clone(),
            hyle_hyllar::HyllarAction::TransferFrom {
                owner: owner.to_string(),
                recipient: recipient.to_string(),
                amount,
            },
        )
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Token {
    pub total_supply: u128,
    pub balances: HashMap<Identity, u128>,
    pub allowances: HashMap<(Identity, Identity), u128>,
}

/// Enum representing possible calls to the token contract functions.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenAction {
    Mint { to: Identity, amount: u128 },
    Burn { from: Identity, amount: u128 },
    Transfer { to: Identity, amount: u128 },
    Approve { spender: Identity, amount: u128 },
    TransferFrom { from: Identity, to: Identity, amount: u128 },
}

impl TokenAction {
    pub fn as_blob(&self, contract_name: sdk::ContractName) -> sdk::Blob {
        sdk::Blob {
            contract_name,
            data: sdk::BlobData(borsh::to_vec(self).expect("Failed to encode TokenAction")),
        }
    }
}

impl Token {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl From<sdk::StateCommitment> for Token {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode token state".to_string())
            .unwrap()
    }
} 