#[allow(unused)]
#[cfg(all(not(clippy), feature = "nonreproducible"))]
mod methods {
    include!(concat!(env!("OUT_DIR"), "/methods.rs"));
}

#[cfg(all(not(clippy), feature = "nonreproducible", feature = "all"))]
mod metadata {
    pub const CONTRACT1_ELF: &[u8] = crate::methods::CONTRACT1_ELF;
    pub const CONTRACT1_ID: [u8; 32] = sdk::to_u8_array(&crate::methods::CONTRACT1_ID);

    pub const CONTRACT2_ELF: &[u8] = crate::methods::CONTRACT2_ELF;
    pub const CONTRACT2_ID: [u8; 32] = sdk::to_u8_array(&crate::methods::CONTRACT2_ID);

    pub const TOKEN_ELF: &[u8] = crate::methods::TOKEN_ELF;
    pub const TOKEN_ID: [u8; 32] = sdk::to_u8_array(&crate::methods::TOKEN_ID);

    pub const AMM_ELF: &[u8] = crate::methods::AMM_ELF;
    pub const AMM_ID: [u8; 32] = sdk::to_u8_array(&crate::methods::AMM_ID);
}

#[cfg(any(clippy, not(feature = "nonreproducible")))]
mod metadata {
    pub const CONTRACT1_ELF: &[u8] =
        contract1::client::tx_executor_handler::metadata::CONTRACT1_ELF;
    pub const CONTRACT1_ID: [u8; 32] = contract1::client::tx_executor_handler::metadata::PROGRAM_ID;

    pub const CONTRACT2_ELF: &[u8] =
        contract2::client::tx_executor_handler::metadata::CONTRACT2_ELF;
    pub const CONTRACT2_ID: [u8; 32] = contract2::client::tx_executor_handler::metadata::PROGRAM_ID;

    pub const TOKEN_ELF: &[u8] =
        token::client::tx_executor_handler::metadata::TOKEN_ELF;
    pub const TOKEN_ID: [u8; 32] = token::client::tx_executor_handler::metadata::PROGRAM_ID;

    pub const AMM_ELF: &[u8] =
        amm::client::tx_executor_handler::metadata::AMM_ELF;
    pub const AMM_ID: [u8; 32] = amm::client::tx_executor_handler::metadata::PROGRAM_ID;
}

pub use metadata::*;
