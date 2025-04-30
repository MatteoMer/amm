use client_sdk::transaction_builder::TxExecutorHandler;
use sdk::{utils::as_hyle_output, Blob, Calldata, ZkContract};

use crate::Amm;

pub mod metadata {
    pub const AMM_ELF: &[u8] = include_bytes!("../../amm.img");
    pub const PROGRAM_ID: [u8; 32] = sdk::str_to_u8(include_str!("../../amm.txt"));
}

impl TxExecutorHandler for Amm {
    fn build_commitment_metadata(&self, _blob: &Blob) -> Result<Vec<u8>, String> {
        borsh::to_vec(self).map_err(|e| e.to_string())
    }

    fn handle(&mut self, calldata: &Calldata) -> Result<sdk::HyleOutput, String> {
        let initial_state_commitment = <Self as ZkContract>::commit(self);
        let mut res = <Self as ZkContract>::execute(self, calldata);
        let next_state_commitment = <Self as ZkContract>::commit(self);
        Ok(as_hyle_output(
            initial_state_commitment,
            next_state_commitment,
            calldata,
            &mut res,
        ))
    }
} 