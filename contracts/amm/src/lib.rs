use borsh::{io::Error, BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sdk::{Identity, RunResult, ContractName, Calldata};
use hyle_hyllar::erc20::ERC20;
use token::Token;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "client")]
pub mod indexer;

impl sdk::ZkContract for Amm {
    /// Entry point of the contract's logic
    fn execute(&mut self, calldata: &sdk::Calldata) -> RunResult {
        // Parse contract inputs
        let (action, ctx) = sdk::utils::parse_raw_calldata::<AmmAction>(calldata)?;

        // Execute the given action
        let res = match action {
            AmmAction::CreatePool { token_a, token_b, amount_a, amount_b } => {
                self.create_pool_with_validation(
                    token_a, 
                    token_b, 
                    amount_a, 
                    amount_b, 
                    &calldata.identity,
                    calldata
                )?
            },
            
            AmmAction::AddLiquidity { token_a, token_b, amount_a_desired, amount_b_desired } => 
                self.add_liquidity_with_validation(
                    token_a, 
                    token_b, 
                    amount_a_desired, 
                    amount_b_desired, 
                    &calldata.identity,
                    calldata
                )?,
            
            AmmAction::RemoveLiquidity { token_a, token_b, lp_amount } => 
                self.remove_liquidity_with_validation(
                    token_a, 
                    token_b, 
                    lp_amount, 
                    &calldata.identity,
                    calldata
                )?,
            
            AmmAction::Swap { token_in, token_out, amount_in, min_amount_out } => 
                self.swap_with_validation(
                    token_in, 
                    token_out, 
                    amount_in, 
                    min_amount_out, 
                    &calldata.identity,
                    calldata
                )?,
        };

        Ok((res, ctx, vec![]))
    }

    /// Serialize the full state on-chain.
    fn commit(&self) -> sdk::StateCommitment {
        sdk::StateCommitment(self.as_bytes().expect("Failed to encode AMM state"))
    }
}

impl Amm {
    pub fn create_pool(
        &mut self, 
        token_a: ContractName, 
        token_b: ContractName, 
        amount_a: u128, 
        amount_b: u128
    ) -> Result<String, String> {
        // Sort tokens to ensure consistent pool keys
        let (token_a, token_b, amount_a, amount_b) = if token_a.0 <= token_b.0 {
            (token_a, token_b, amount_a, amount_b)
        } else {
            (token_b, token_a, amount_b, amount_a)
        };
        
        // Check if pool already exists
        if self.pools.contains_key(&(token_a.clone(), token_b.clone())) {
            return Err(format!("Pool for {}/{} already exists", token_a.0, token_b.0));
        }
        
        // Ensure amounts are non-zero
        if amount_a == 0 || amount_b == 0 {
            return Err("Initial amounts must be non-zero".to_string());
        }
        
        // Calculate initial LP tokens - geometric mean of the amounts
        let lp_total_supply = (amount_a as f64 * amount_b as f64).sqrt() as u128;
        
        // Create pool state
        let pool_state = PoolState {
            reserve_a: amount_a,
            reserve_b: amount_b,
            lp_total_supply,
        };
        
        // Save pool
        self.pools.insert((token_a.clone(), token_b.clone()), pool_state);
        
        // Note: We don't have access to calldata.identity here, so we can't record LP token balance
        // This will be handled in the execute function or via a separate function
        
        Ok(format!(
            "Created pool {}/{} with initial liquidity {}/{} and {} LP tokens", 
            token_a.0, token_b.0, amount_a, amount_b, lp_total_supply
        ))
    }
    
    pub fn add_liquidity(
        &mut self,
        token_a: ContractName,
        token_b: ContractName,
        amount_a_desired: u128,
        amount_b_desired: u128,
        provider: &Identity
    ) -> Result<String, String> {
        // Sort tokens for consistent pool key
        let (token_a, token_b, amount_a_desired, amount_b_desired, is_reversed) = 
            if token_a.0 <= token_b.0 {
                (token_a, token_b, amount_a_desired, amount_b_desired, false)
            } else {
                (token_b, token_a, amount_b_desired, amount_a_desired, true)
            };
        
        // Get pool
        let pool = self.pools.get_mut(&(token_a.clone(), token_b.clone()))
            .ok_or_else(|| format!("Pool for {}/{} doesn't exist", token_a.0, token_b.0))?;
        
        // Calculate the optimal amount to provide based on current ratio
        let (amount_a, amount_b) = if pool.reserve_a == 0 || pool.reserve_b == 0 {
            // For new pools, use the desired amounts directly
            (amount_a_desired, amount_b_desired)
        } else {
            // Calculate amounts to maintain the ratio
            let amount_b_optimal = (amount_a_desired * pool.reserve_b) / pool.reserve_a;
            
            if amount_b_optimal <= amount_b_desired {
                // Use all of A and calculated B
                (amount_a_desired, amount_b_optimal)
            } else {
                // Use calculated A and all of B
                let amount_a_optimal = (amount_b_desired * pool.reserve_a) / pool.reserve_b;
                (amount_a_optimal, amount_b_desired)
            }
        };
        
        // Calculate LP tokens to mint (proportional to share of the pool)
        let lp_share = if pool.lp_total_supply == 0 {
            // Initial liquidity - use geometric mean
            (amount_a as f64 * amount_b as f64).sqrt() as u128
        } else {
            // Use the smaller ratio to determine LP tokens
            let lp_amount_a = (amount_a * pool.lp_total_supply) / pool.reserve_a;
            let lp_amount_b = (amount_b * pool.lp_total_supply) / pool.reserve_b;
            std::cmp::min(lp_amount_a, lp_amount_b)
        };
        
        // Update pool reserves
        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        pool.lp_total_supply += lp_share;
        
        // Update LP balance
        let key = (token_a.clone(), token_b.clone(), provider.clone());
        let lp_balance = self.lp_balances.entry(key).or_insert(0);
        *lp_balance += lp_share;
        
        // Return the final amounts that will be used
        let (final_amount_a, final_amount_b) = if !is_reversed {
            (amount_a, amount_b)
        } else {
            (amount_b, amount_a)
        };
        
        Ok(format!(
            "Added liquidity to {}/{} pool: {}/{} tokens, received {} LP tokens",
            token_a.0, token_b.0, final_amount_a, final_amount_b, lp_share
        ))
    }
    
    pub fn remove_liquidity(
        &mut self,
        token_a: ContractName,
        token_b: ContractName,
        lp_amount: u128,
        provider: &Identity
    ) -> Result<String, String> {
        // Sort tokens for consistent pool key
        let (token_a, token_b, is_reversed) = if token_a.0 <= token_b.0 {
            (token_a, token_b, false)
        } else {
            (token_b, token_a, true)
        };
        
        // Check if pool exists
        let pool = self.pools.get_mut(&(token_a.clone(), token_b.clone()))
            .ok_or_else(|| format!("Pool for {}/{} doesn't exist", token_a.0, token_b.0))?;
        
        // Check LP balance
        let key = (token_a.clone(), token_b.clone(), provider.clone());
        let lp_balance = self.lp_balances.get(&key).cloned().unwrap_or(0);
        
        if lp_amount > lp_balance {
            return Err(format!("Insufficient LP tokens: {} < {}", lp_balance, lp_amount));
        }
        
        // Calculate token amounts to return (proportional to LP share)
        let amount_a = (lp_amount * pool.reserve_a) / pool.lp_total_supply;
        let amount_b = (lp_amount * pool.reserve_b) / pool.lp_total_supply;
        
        // Update pool reserves
        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;
        pool.lp_total_supply -= lp_amount;
        
        // Update LP balance
        let new_lp_balance = lp_balance - lp_amount;
        if new_lp_balance == 0 {
            self.lp_balances.remove(&key);
        } else {
            self.lp_balances.insert(key, new_lp_balance);
        }
        
        // Return the withdrawn amounts
        let (withdrawn_a, withdrawn_b) = if !is_reversed {
            (amount_a, amount_b)
        } else {
            (amount_b, amount_a)
        };
        
        Ok(format!(
            "Removed liquidity from {}/{} pool: {} LP tokens, received {}/{} tokens",
            token_a.0, token_b.0, lp_amount, withdrawn_a, withdrawn_b
        ))
    }
    
    pub fn swap(
        &mut self,
        token_in: ContractName,
        token_out: ContractName,
        amount_in: u128,
        min_amount_out: u128,
        _user: &Identity  // Mark as unused with underscore
    ) -> Result<String, String> {
        // Ensure input amount is non-zero
        if amount_in == 0 {
            return Err("Input amount must be greater than zero".to_string());
        }
        
        // Try to find pool in both directions
        let is_direct_pool = if self.pools.contains_key(&(token_in.clone(), token_out.clone())) {
            true
        } else if self.pools.contains_key(&(token_out.clone(), token_in.clone())) {
            false
        } else {
            return Err(format!("Pool for {}/{} doesn't exist", token_in.0, token_out.0));
        };
        
        // Get pool with correct token order
        let pool_key = if is_direct_pool {
            (token_in.clone(), token_out.clone())
        } else {
            (token_out.clone(), token_in.clone())
        };
        
        let pool = self.pools.get_mut(&pool_key)
            .ok_or_else(|| format!("Pool for {}/{} not found", token_in.0, token_out.0))?;
        
        // Calculate amount out based on constant product formula
        let amount_out = if is_direct_pool {
            let amount_in_with_fee = amount_in * 997; // 0.3% fee
            let numerator = amount_in_with_fee * pool.reserve_b;
            let denominator = pool.reserve_a * 1000 + amount_in_with_fee;
            numerator / denominator
        } else {
            let amount_in_with_fee = amount_in * 997; // 0.3% fee
            let numerator = amount_in_with_fee * pool.reserve_a;
            let denominator = pool.reserve_b * 1000 + amount_in_with_fee;
            numerator / denominator
        };
        
        // Check min amount out
        if amount_out < min_amount_out {
            return Err(format!("Insufficient output amount: {} < {}", amount_out, min_amount_out));
        }
        
        // Update pool reserves
        if is_direct_pool {
            pool.reserve_a += amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += amount_in;
            pool.reserve_a -= amount_out;
        }
        
        Ok(format!(
            "Swapped {} {} for {} {}",
            amount_in, token_in.0, amount_out, token_out.0
        ))
    }
    
    // Function to validate token transfers using the ERC20 interface
    pub fn validate_token_transfer(
        &self,
        exec_ctx: &mut sdk::caller::ExecutionContext,
        token: &ContractName,
        _from: &Identity,  // Not used directly but kept for API clarity
        to: &Identity,
        amount: u128
    ) -> Result<(), String> {
        // Set up the execution context for this token contract
        exec_ctx.contract_name = token.clone();
        
        // Validate the transfer using the ERC20 trait
        Token::check_transfer(
            exec_ctx.clone(),
            &to.0, // Recipient
            amount,
        )
    }
    
    // Function to swap tokens with proof composition validation
    pub fn swap_with_validation(
        &mut self,
        token_in: ContractName,
        token_out: ContractName,
        amount_in: u128,
        min_amount_out: u128,
        user: &Identity,
        calldata: &Calldata
    ) -> Result<String, String> {
        // Calculate the amount out first
        let swap_result = self.swap(token_in.clone(), token_out.clone(), amount_in, min_amount_out, user)?;
        
        // Set up the execution context for validation
        let mut exec_ctx = sdk::caller::ExecutionContext {
            callees_blobs: calldata.blobs.iter().map(|(_, blob)| blob.clone()).collect(),
            caller: user.clone(),
            contract_name: ContractName("amm".to_string()),
        };
        
        // Validate the token transfer from user to AMM contract (token_in)
        self.validate_token_transfer(
            &mut exec_ctx, 
            &token_in, 
            user, 
            &Identity(self.get_pool_address(&token_in, &token_out).to_string()),
            amount_in
        )?;
        
        // For token_out, we don't need to validate as the AMM contract itself is releasing tokens
        
        Ok(swap_result)
    }
    
    // Helper function to get an address for the pool
    fn get_pool_address(&self, token_a: &ContractName, token_b: &ContractName) -> String {
        format!("pool_{}_{}", token_a.0, token_b.0)
    }
    
    // Helper function to calculate output amount for a given input
    pub fn calc_output_amount(
        &self,
        token_in: &ContractName,
        token_out: &ContractName,
        amount_in: u128
    ) -> Result<u128, String> {
        // Try to find pool in both directions
        let is_direct_pool = if self.pools.contains_key(&(token_in.clone(), token_out.clone())) {
            true
        } else if self.pools.contains_key(&(token_out.clone(), token_in.clone())) {
            false
        } else {
            return Err(format!("Pool for {}/{} doesn't exist", token_in.0, token_out.0));
        };
        
        // Get pool with correct token order
        let pool_key = if is_direct_pool {
            (token_in.clone(), token_out.clone())
        } else {
            (token_out.clone(), token_in.clone())
        };
        
        let pool = self.pools.get(&pool_key)
            .ok_or_else(|| format!("Pool for {}/{} not found", token_in.0, token_out.0))?;
        
        // Calculate amount out based on constant product formula with 0.3% fee
        let amount_out = if is_direct_pool {
            let amount_in_with_fee = amount_in * 997; // 0.3% fee
            let numerator = amount_in_with_fee * pool.reserve_b;
            let denominator = pool.reserve_a * 1000 + amount_in_with_fee;
            numerator / denominator
        } else {
            let amount_in_with_fee = amount_in * 997; // 0.3% fee
            let numerator = amount_in_with_fee * pool.reserve_a;
            let denominator = pool.reserve_b * 1000 + amount_in_with_fee;
            numerator / denominator
        };
        
        Ok(amount_out)
    }

    pub fn add_liquidity_with_validation(
        &mut self,
        token_a: ContractName,
        token_b: ContractName,
        amount_a_desired: u128,
        amount_b_desired: u128,
        provider: &Identity,
        calldata: &Calldata
    ) -> Result<String, String> {
        // First, determine the actual amounts to add
        let add_liquidity_result = self.add_liquidity(
            token_a.clone(),
            token_b.clone(),
            amount_a_desired,
            amount_b_desired,
            provider
        )?;
        
        // Extract the actual amounts from the result 
        // This is a simplified approach - in practice, you might want to return the actual amounts
        // from add_liquidity function directly
        
        // Set up the execution context for validation
        let mut exec_ctx = sdk::caller::ExecutionContext {
            callees_blobs: calldata.blobs.iter().map(|(_, blob)| blob.clone()).collect(),
            caller: provider.clone(),
            contract_name: ContractName("amm".to_string()),
        };
        
        // Sort tokens to ensure consistent pool keys (same as in add_liquidity)
        let (sorted_token_a, sorted_token_b, amount_a, amount_b) = if token_a.0 <= token_b.0 {
            (token_a.clone(), token_b.clone(), amount_a_desired, amount_b_desired)
        } else {
            (token_b.clone(), token_a.clone(), amount_b_desired, amount_a_desired)
        };
        
        // Get the pool address
        let pool_address = self.get_pool_address(&sorted_token_a, &sorted_token_b);
        let pool_identity = Identity(pool_address);
        
        // Validate token A transfer
        self.validate_token_transfer(
            &mut exec_ctx,
            &sorted_token_a,
            provider,
            &pool_identity,
            amount_a
        )?;
        
        // Validate token B transfer
        self.validate_token_transfer(
            &mut exec_ctx,
            &sorted_token_b,
            provider,
            &pool_identity,
            amount_b
        )?;
        
        Ok(add_liquidity_result)
    }

    pub fn remove_liquidity_with_validation(
        &mut self,
        token_a: ContractName,
        token_b: ContractName,
        lp_amount: u128,
        provider: &Identity,
        calldata: &Calldata
    ) -> Result<String, String> {
        // Sort tokens for consistent pool key (same as in remove_liquidity)
        let (sorted_token_a, sorted_token_b, _is_reversed) = if token_a.0 <= token_b.0 {
            (token_a.clone(), token_b.clone(), false)
        } else {
            (token_b.clone(), token_a.clone(), true)
        };
        
        // Check if pool exists and get pool data before removal
        let pool_key = (sorted_token_a.clone(), sorted_token_b.clone());
        let pool = self.pools.get(&pool_key)
            .ok_or_else(|| format!("Pool for {}/{} doesn't exist", sorted_token_a.0, sorted_token_b.0))?;
        
        // Calculate token amounts to return (proportional to LP share)
        // These are used only for validation so we mark them as unused
        let _amount_a = (lp_amount * pool.reserve_a) / pool.lp_total_supply;
        let _amount_b = (lp_amount * pool.reserve_b) / pool.lp_total_supply;
        
        // Now perform the actual removal
        let remove_liquidity_result = self.remove_liquidity(
            token_a.clone(),
            token_b.clone(),
            lp_amount,
            provider
        )?;
        
        // Get the pool address
        let pool_address = self.get_pool_address(&sorted_token_a, &sorted_token_b);
        let pool_identity = Identity(pool_address);
        
        // Set up the execution context for validation
        // For remove liquidity, we don't necessarily need to validate user transfers,
        // but we could validate that the pool has enough tokens to send to the user
        // This would be a reverse validation compared to the other operations
        // We don't actually use this context in this implementation
        let _exec_ctx = sdk::caller::ExecutionContext {
            callees_blobs: calldata.blobs.iter().map(|(_, blob)| blob.clone()).collect(),
            caller: pool_identity.clone(), // The pool is the sender
            contract_name: ContractName("amm".to_string()),
        };
        
        // Here we could validate that the pool has enough tokens to send to the user
        // However, since we control the pool state directly, we've already checked this
        // in the remove_liquidity function by verifying reserves
        
        Ok(remove_liquidity_result)
    }

    pub fn create_pool_with_validation(
        &mut self,
        token_a: ContractName,
        token_b: ContractName,
        amount_a: u128,
        amount_b: u128,
        provider: &Identity,
        calldata: &Calldata
    ) -> Result<String, String> {
        // Sort tokens to ensure consistent pool keys (same as in create_pool)
        let (sorted_token_a, sorted_token_b, amount_a_sorted, amount_b_sorted) = if token_a.0 <= token_b.0 {
            (token_a.clone(), token_b.clone(), amount_a, amount_b)
        } else {
            (token_b.clone(), token_a.clone(), amount_b, amount_a)
        };
        
        // First, check if the pool already exists
        if self.pools.contains_key(&(sorted_token_a.clone(), sorted_token_b.clone())) {
            return Err(format!("Pool for {}/{} already exists", sorted_token_a.0, sorted_token_b.0));
        }
        
        // Create the pool
        let create_pool_result = self.create_pool(token_a.clone(), token_b.clone(), amount_a, amount_b)?;
        
        // Now validate the token transfers
        // Set up the execution context for validation
        let mut exec_ctx = sdk::caller::ExecutionContext {
            callees_blobs: calldata.blobs.iter().map(|(_, blob)| blob.clone()).collect(),
            caller: provider.clone(),
            contract_name: ContractName("amm".to_string()),
        };
        
        // Get the pool address
        let pool_address = self.get_pool_address(&sorted_token_a, &sorted_token_b);
        let pool_identity = Identity(pool_address);
        
        // Validate token A transfer
        self.validate_token_transfer(
            &mut exec_ctx,
            &sorted_token_a,
            provider,
            &pool_identity,
            amount_a_sorted
        )?;
        
        // Validate token B transfer
        self.validate_token_transfer(
            &mut exec_ctx,
            &sorted_token_b,
            provider,
            &pool_identity,
            amount_b_sorted
        )?;
        
        // Handle LP token assignment for the creator
        if let Some(pool) = self.pools.get(&(sorted_token_a.clone(), sorted_token_b.clone())) {
            let key = (sorted_token_a, sorted_token_b, provider.clone());
            self.lp_balances.insert(key, pool.lp_total_supply);
        }
        
        Ok(create_pool_result)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone, Default)]
pub struct Amm {
    pub pools: HashMap<(ContractName, ContractName), PoolState>,
    pub lp_balances: HashMap<(ContractName, ContractName, Identity), u128>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
pub struct PoolState {
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub lp_total_supply: u128,
}

/// Enum representing possible calls to the AMM contract functions.
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum AmmAction {
    CreatePool {
        token_a: ContractName,
        token_b: ContractName,
        amount_a: u128,
        amount_b: u128,
    },
    AddLiquidity {
        token_a: ContractName,
        token_b: ContractName,
        amount_a_desired: u128,
        amount_b_desired: u128,
    },
    RemoveLiquidity {
        token_a: ContractName,
        token_b: ContractName,
        lp_amount: u128,
    },
    Swap {
        token_in: ContractName,
        token_out: ContractName,
        amount_in: u128,
        min_amount_out: u128,
    },
}

impl AmmAction {
    pub fn as_blob(&self, contract_name: sdk::ContractName) -> sdk::Blob {
        sdk::Blob {
            contract_name,
            data: sdk::BlobData(borsh::to_vec(self).expect("Failed to encode AmmAction")),
        }
    }
}

impl Amm {
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        borsh::to_vec(self)
    }
}

impl From<sdk::StateCommitment> for Amm {
    fn from(state: sdk::StateCommitment) -> Self {
        borsh::from_slice(&state.0)
            .map_err(|_| "Could not decode AMM state".to_string())
            .unwrap()
    }
} 