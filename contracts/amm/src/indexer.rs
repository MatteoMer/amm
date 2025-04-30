use std::str;

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
use sdk::{Hashed, ContractName};
use serde::Serialize;
use tracing;

use crate::*;
use client_sdk::contract_indexer::axum;
use client_sdk::contract_indexer::utoipa;

impl ContractHandler for Amm {
    async fn api(store: ContractHandlerStore<Amm>) -> (Router<()>, OpenApi) {
        let (router, api) = OpenApiRouter::default()
            .routes(routes!(get_state, get_pool, get_price))
            .split_for_parts();

        (router.with_state(store), api)
    }

    fn handle_transaction(
        &mut self,
        tx: &sdk::BlobTransaction,
        index: sdk::BlobIndex,
        tx_context: sdk::TxContext,
    ) -> Result<()> {
        let sdk::Blob {
            contract_name,
            data: _,
        } = tx.blobs.get(index.0).context("Failed to get blob")?;

        let calldata = sdk::Calldata {
            identity: tx.identity.clone(),
            index,
            blobs: tx.blobs.clone().into(),
            tx_blob_count: tx.blobs.len(),
            tx_hash: tx.hashed(),
            tx_ctx: Some(tx_context),
            private_input: vec![],
        };

        let hyle_output = self.handle(&calldata).map_err(|e| anyhow::anyhow!(e))?;
        let program_outputs = str::from_utf8(&hyle_output.program_outputs).unwrap_or("no output");

        sdk::info!("🚀 Executed {contract_name}: {}", program_outputs);
        tracing::debug!(
            handler = %contract_name,
            "hyle_output: {:?}", hyle_output
        );
        Ok(())
    }
}

#[utoipa::path(
    get,
    path = "/state",
    tag = "AMM",
    responses(
        (status = OK, description = "Get json state of AMM contract")
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

#[utoipa::path(
    get,
    path = "/pool/{token_a}/{token_b}",
    tag = "AMM",
    responses(
        (status = OK, description = "Get pool information for a token pair")
    )
)]
pub async fn get_pool(
    State(state): State<ContractHandlerStore<Amm>>,
    Path((token_a, token_b)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let store = state.read().await;
    
    let amm = store.state.as_ref().ok_or(AppError(
        StatusCode::NOT_FOUND,
        anyhow!("No state found for contract '{}'", store.contract_name),
    ))?;
    
    let token_a = ContractName(token_a);
    let token_b = ContractName(token_b);
    
    // Try both combinations since the order might be different
    let pool = amm.pools.get(&(token_a.clone(), token_b.clone()))
        .or_else(|| amm.pools.get(&(token_b.clone(), token_a.clone())))
        .ok_or(AppError(
            StatusCode::NOT_FOUND,
            anyhow!("Pool for {}/{} not found", token_a.0, token_b.0),
        ))?
        .clone();
    
    Ok(Json(pool))
}

#[utoipa::path(
    get,
    path = "/price/{token_in}/{token_out}",
    tag = "AMM",
    responses(
        (status = OK, description = "Get price information for a token pair")
    )
)]
pub async fn get_price(
    State(state): State<ContractHandlerStore<Amm>>,
    Path((token_in, token_out)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let store = state.read().await;
    
    let amm = store.state.as_ref().ok_or(AppError(
        StatusCode::NOT_FOUND,
        anyhow!("No state found for contract '{}'", store.contract_name),
    ))?;
    
    let token_in = ContractName(token_in);
    let token_out = ContractName(token_out);
    
    // Check direct pool
    let direct_pool = amm.pools.get(&(token_in.clone(), token_out.clone()));
    
    // Check reverse pool
    let reverse_pool = amm.pools.get(&(token_out.clone(), token_in.clone()));
    
    // Determine the pool and direction
    let (pool, is_reversed) = match (direct_pool, reverse_pool) {
        (Some(pool), _) => (pool, false),
        (_, Some(pool)) => (pool, true),
        _ => return Err(AppError(
            StatusCode::NOT_FOUND,
            anyhow!("Pool for {}/{} not found", token_in.0, token_out.0),
        )),
    };
    
    // Calculate the price for 1 token_in in terms of token_out
    let price = if is_reversed {
        // For reversed pool, we need to swap the reserves
        if pool.reserve_b == 0 {
            return Err(AppError(
                StatusCode::BAD_REQUEST,
                anyhow!("Zero liquidity for {}", token_in.0),
            ));
        }
        pool.reserve_a as f64 / pool.reserve_b as f64
    } else {
        // For direct pool
        if pool.reserve_a == 0 {
            return Err(AppError(
                StatusCode::BAD_REQUEST,
                anyhow!("Zero liquidity for {}", token_in.0),
            ));
        }
        pool.reserve_b as f64 / pool.reserve_a as f64
    };
    
    #[derive(Serialize)]
    struct PriceInfo {
        price: f64,
        reserve_in: u128,
        reserve_out: u128,
    }
    
    let (reserve_in, reserve_out) = if is_reversed {
        (pool.reserve_b, pool.reserve_a)
    } else {
        (pool.reserve_a, pool.reserve_b)
    };
    
    Ok(Json(PriceInfo {
        price,
        reserve_in,
        reserve_out,
    }))
} 