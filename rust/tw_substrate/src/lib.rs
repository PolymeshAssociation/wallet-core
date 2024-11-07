use tw_coin_entry::error::prelude::*;
use tw_ss58_address::NetworkId;

pub mod address;
pub use address::*;

pub mod extrinsic;
pub use extrinsic::*;

pub mod extensions;
pub use extensions::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EncodeError {
    InvalidNetworkId,
    MissingCallIndicesTable,
    InvalidCallIndex,
    InvalidAddress,
    InvalidValue,
}

impl From<EncodeError> for SigningError {
    #[inline]
    fn from(_err: EncodeError) -> Self {
        TWError::new(SigningErrorType::Error_invalid_params)
    }
}

pub type EncodeResult<T> = Result<T, EncodeError>;
pub type WithCallIndexResult<T> = Result<WithCallIndex<T>, EncodeError>;

#[derive(Debug, Clone)]
pub struct SubstrateContext {
    pub multi_address: bool,
    pub network: NetworkId,
    pub spec_version: u32,
    pub transaction_version: u32,
}

impl SubstrateContext {
    pub fn multi_address(&self, account: AccountId) -> MultiAddress {
        MultiAddress::new(account, self.multi_address)
    }

    pub fn multi_addresses(&self, accounts: Vec<AccountId>) -> Vec<MultiAddress> {
        accounts
            .into_iter()
            .map(|account| MultiAddress::new(account, self.multi_address))
            .collect()
    }
}
