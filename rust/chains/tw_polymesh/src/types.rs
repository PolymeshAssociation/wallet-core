use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use tw_coin_entry::error::prelude::TWError;
use tw_hash::{Hash, H256};
use tw_proto::Polymesh::Proto::{
    mod_SecondaryKeyPermissions::{
        AssetPermissions as TWAssetPermissions, ExtrinsicPermissions as TWExtrinsicPermissions,
        PalletPermissions as TWPalletPermissions, PortfolioPermissions as TWPortfolioPermissions,
        RestrictionKind as TWRestrictionKind,
    },
    AssetId as TWAssetId, IdentityId as TWIdentityId, PortfolioId as TWPortfolioId,
    SecondaryKeyPermissions,
};
use tw_scale::{impl_enum_scale, impl_struct_scale, ToScale};
use tw_ss58_address::SS58Address;
use tw_substrate::address::SubstrateAddress;

use super::*;

impl_struct_scale!(
    #[derive(Clone, Debug)]
    pub struct Memo(H256);
);

impl Memo {
    pub fn new(memo: &str) -> Self {
        let memo = memo.as_bytes();
        let mut bytes = [0; 32];
        let len = memo.len().min(32);
        bytes[0..len].copy_from_slice(&memo[0..len]);

        Self(bytes.into())
    }
}

pub type H128 = Hash<16>;

impl_struct_scale!(
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct AssetId(H128);
);

impl TryFrom<&TWAssetId<'_>> for AssetId {
    type Error = TWError<EncodeError>;

    fn try_from(id: &TWAssetId) -> Result<Self, Self::Error> {
        let did = H128::try_from(id.id.as_ref()).map_err(|_| {
            EncodeError::InvalidValue.with_context(format!("Expected 16 byte AssetId"))
        })?;
        Ok(Self(did))
    }
}

impl_struct_scale!(
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct IdentityId(H256);
);

impl TryFrom<&TWIdentityId<'_>> for IdentityId {
    type Error = TWError<EncodeError>;

    fn try_from(id: &TWIdentityId) -> Result<Self, Self::Error> {
        let did = H256::try_from(id.id.as_ref()).map_err(|_| {
            EncodeError::InvalidValue.with_context(format!("Expected 32 byte IdentityId"))
        })?;
        Ok(Self(did))
    }
}

impl_enum_scale!(
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PortfolioKind {
        #[default]
        Default = 0x00,
        User(u64) = 0x01,
    }
);

impl_struct_scale!(
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct PortfolioId {
        did: IdentityId,
        kind: PortfolioKind,
    }
);

impl TryFrom<&TWPortfolioId<'_>> for PortfolioId {
    type Error = TWError<EncodeError>;

    fn try_from(portfolio: &TWPortfolioId) -> Result<Self, Self::Error> {
        Ok(Self {
            did: portfolio
                .identity
                .as_ref()
                .ok_or_else(|| {
                    EncodeError::InvalidValue.with_context(format!("Missing portfolio identity"))
                })?
                .try_into()?,
            kind: if portfolio.default {
                PortfolioKind::Default
            } else {
                PortfolioKind::User(portfolio.user)
            },
        })
    }
}

impl_enum_scale!(
    #[derive(Clone, Debug)]
    pub enum Signatory {
        Identity(IdentityId) = 0x00,
        Account(AccountId) = 0x01,
    }
);

impl_enum_scale!(
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub enum RestrictionKind {
        #[default]
        Whole = 0x00,
        These = 0x01,
        Except = 0x02,
    }
);

impl From<TWRestrictionKind> for RestrictionKind {
    fn from(kind: TWRestrictionKind) -> Self {
        match kind {
            TWRestrictionKind::Whole => Self::Whole,
            TWRestrictionKind::These => Self::These,
            TWRestrictionKind::Except => Self::Except,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AssetPermissions {
    kind: RestrictionKind,
    assets: BTreeSet<AssetId>,
}

impl AssetPermissions {
    /// Empty permissions means no access.
    pub fn empty() -> Self {
        Self {
            kind: RestrictionKind::These,
            assets: BTreeSet::new(),
        }
    }
}

impl ToScale for AssetPermissions {
    fn to_scale_into(&self, data: &mut Vec<u8>) {
        self.kind.to_scale_into(data);
        if self.kind != RestrictionKind::Whole {
            self.assets.to_scale_into(data);
        }
    }
}

impl TryFrom<&TWAssetPermissions<'_>> for AssetPermissions {
    type Error = TWError<EncodeError>;

    fn try_from(perms: &TWAssetPermissions) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: perms.kind.into(),
            assets: perms
                .assets
                .iter()
                .map(|asset| Ok(asset.try_into()?))
                .collect::<EncodeResult<BTreeSet<AssetId>>>()?,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PortfolioPermissions {
    kind: RestrictionKind,
    portfolios: BTreeSet<PortfolioId>,
}

impl PortfolioPermissions {
    /// Empty permissions means no access.
    pub fn empty() -> Self {
        Self {
            kind: RestrictionKind::These,
            portfolios: BTreeSet::new(),
        }
    }
}

impl ToScale for PortfolioPermissions {
    fn to_scale_into(&self, data: &mut Vec<u8>) {
        self.kind.to_scale_into(data);
        if self.kind != RestrictionKind::Whole {
            self.portfolios.to_scale_into(data);
        }
    }
}

impl TryFrom<&TWPortfolioPermissions<'_>> for PortfolioPermissions {
    type Error = TWError<EncodeError>;

    fn try_from(perms: &TWPortfolioPermissions) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: perms.kind.into(),
            portfolios: perms
                .portfolios
                .iter()
                .map(|portfolio| Ok(portfolio.try_into()?))
                .collect::<EncodeResult<BTreeSet<PortfolioId>>>()?,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct PalletPermissions {
    kind: RestrictionKind,
    extrinsic_names: BTreeSet<String>,
}

impl ToScale for PalletPermissions {
    fn to_scale_into(&self, data: &mut Vec<u8>) {
        self.kind.to_scale_into(data);
        if self.kind != RestrictionKind::Whole {
            self.extrinsic_names.to_scale_into(data);
        }
    }
}

impl From<&TWPalletPermissions<'_>> for PalletPermissions {
    fn from(perms: &TWPalletPermissions) -> Self {
        Self {
            kind: perms.kind.into(),
            extrinsic_names: perms
                .extrinsic_names
                .iter()
                .map(|name| name.to_string())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExtrinsicPermissions {
    kind: RestrictionKind,
    pallets: BTreeMap<String, PalletPermissions>,
}

impl ExtrinsicPermissions {
    /// Empty permissions means no access.
    pub fn empty() -> Self {
        Self {
            kind: RestrictionKind::These,
            pallets: BTreeMap::new(),
        }
    }
}

impl ToScale for ExtrinsicPermissions {
    fn to_scale_into(&self, data: &mut Vec<u8>) {
        self.kind.to_scale_into(data);
        if self.kind != RestrictionKind::Whole {
            self.pallets.to_scale_into(data);
        }
    }
}

impl From<&TWExtrinsicPermissions<'_>> for ExtrinsicPermissions {
    fn from(perms: &TWExtrinsicPermissions) -> Self {
        Self {
            kind: perms.kind.into(),
            pallets: perms
                .pallets
                .iter()
                .map(|pallet| (pallet.pallet_name.to_string(), pallet.into()))
                .collect(),
        }
    }
}

impl_struct_scale!(
    #[derive(Clone, Debug, Default)]
    pub struct Permissions {
        asset: AssetPermissions,
        extrinsic: ExtrinsicPermissions,
        portfolio: PortfolioPermissions,
    }
);

impl TryFrom<&SecondaryKeyPermissions<'_>> for Permissions {
    type Error = TWError<EncodeError>;

    fn try_from(perms: &SecondaryKeyPermissions) -> Result<Self, Self::Error> {
        Ok(Self {
            asset: if let Some(perms) = &perms.asset {
                perms.try_into()?
            } else {
                AssetPermissions::default()
            },
            extrinsic: if let Some(perms) = &perms.extrinsic {
                perms.into()
            } else {
                ExtrinsicPermissions::default()
            },
            portfolio: if let Some(perms) = &perms.portfolio {
                perms.try_into()?
            } else {
                PortfolioPermissions::default()
            },
        })
    }
}

impl_enum_scale!(
    #[derive(Clone, Debug)]
    pub enum AuthorizationData {
        JoinIdentity { permissions: Permissions } = 0x05,
    }
);

impl_enum_scale!(
    #[derive(Clone, Debug)]
    pub enum RewardDestination {
        Staked = 0x00,
        Stash = 0x01,
        Controller = 0x02,
        Account(AccountId) = 0x03,
        None = 0x04,
    }
);

impl RewardDestination {
    pub fn from_tw(dest: u8, account: &str) -> EncodeResult<Self> {
        match dest {
            0 => Ok(Self::Staked),
            1 => Ok(Self::Stash),
            2 => Ok(Self::Controller),
            4 => {
                let account =
                    SS58Address::from_str(account).map_err(|_| EncodeError::InvalidAddress)?;
                Ok(Self::Account(SubstrateAddress(account)))
            },
            5 => Ok(Self::None),
            _ => EncodeError::InvalidValue.tw_result(format!("Invalid reward destination: {dest}")),
        }
    }
}
