use tw_hash::{H256, H512};
use tw_scale::{impl_enum_scale, impl_struct_scale, Compact, ToScale};
use tw_ss58_address::SS58Address;

use tw_proto::Polkadot::Proto::{
    self, mod_CallIndices::OneOfvariant as CallIndicesVariant, CallIndices,
};

use crate::address::PolkadotAddress;
use crate::extrinsic::EncodeError;

pub type TxHash = H256;
pub type BlockHash = H256;
pub type BlockNumber = u32;

pub type AccountId = PolkadotAddress;
pub type AccountIndex = u32;

impl_enum_scale!(
    #[derive(Clone, Debug)]
    pub enum MultiSignature {
        Ed25519(H512),
        Sr25519(H512),
        //Ecdsa([u8; 65]),
    }
);

impl_enum_scale!(
    #[derive(Clone, Debug)]
    pub enum MultiAddress {
        /// It's an account ID (pubkey).
        Id(AccountId),
        // /// It's an account index.
        // Index(Compact<AccountIndex>),
        // /// It's some arbitrary raw bytes.
        // Raw(Vec<u8>),
        // /// It's a 32 byte representation.
        // Address32([u8; 32]),
        // /// Its a 20 byte representation.
        // Address20([u8; 20]),
    }
);

impl From<AccountId> for MultiAddress {
    fn from(other: AccountId) -> Self {
        Self::Id(other)
    }
}

impl From<SS58Address> for MultiAddress {
    fn from(other: SS58Address) -> Self {
        Self::Id(PolkadotAddress(other))
    }
}

impl_struct_scale!(
    #[derive(Clone, Debug, Default)]
    pub struct AdditionalSigned {
        pub spec_version: u32,
        pub tx_version: u32,
        pub genesis_hash: BlockHash,
        pub current_hash: BlockHash,
    }
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Era {
    Immortal,
    Mortal(u64, u64),
}

impl Era {
    pub fn from_tw(era: &Option<Proto::Era>) -> Self {
        match era {
            None => Self::immortal(),
            Some(era) => Self::mortal(era.period, era.block_number),
        }
    }

    pub fn mortal(period: u64, block: u64) -> Self {
        // Based off `sp_runtime::generic::Era`:
        // See https://github.com/paritytech/polkadot-sdk/blob/657b5503a04e97737696fa7344641019350fb521/substrate/primitives/runtime/src/generic/era.rs#L65
        let period = period
            .checked_next_power_of_two()
            .unwrap_or(1 << 16)
            .clamp(4, 1 << 16);
        let phase = block % period;
        let quantize_factor = (period >> 12).max(1);
        let quantized_phase = phase / quantize_factor * quantize_factor;
        Self::Mortal(period, quantized_phase)
    }

    pub fn immortal() -> Self {
        Self::Immortal
    }
}

impl ToScale for Era {
    fn to_scale_into(&self, out: &mut Vec<u8>) {
        match self {
            Self::Immortal => {
                out.push(0);
            },
            Self::Mortal(period, phase) => {
                // Based off `sp_runtime::generic::Era`:
                // See https://github.com/paritytech/polkadot-sdk/blob/657b5503a04e97737696fa7344641019350fb521/substrate/primitives/runtime/src/generic/era.rs#L107
                let quantize_factor = (period >> 12).max(1);

                let encoded = (period.trailing_zeros() - 1).clamp(1, 15) as u16
                    | ((phase / quantize_factor) << 4) as u16;
                encoded.to_scale_into(out);
            },
        }
    }
}

impl_struct_scale!(
    #[derive(Clone, Debug)]
    pub struct Extra {
        era: Era,
        nonce: Compact<u32>,
        tip: Compact<u128>,
    }
);

impl Extra {
    pub fn new(era: Era, nonce: u32) -> Self {
        Self {
            era,
            nonce: Compact(nonce),
            tip: Compact(0u128),
        }
    }

    pub fn nonce(&self) -> u32 {
        self.nonce.0
    }

    pub fn tip(&self) -> u128 {
        self.tip.0
    }
}

#[derive(Clone, Debug)]
pub struct CallIndex(Option<(u8, u8)>);

impl CallIndex {
    pub fn from_tw(call_index: &Option<CallIndices>) -> Result<Self, EncodeError> {
        let call_index = match call_index {
            Some(CallIndices {
                variant: CallIndicesVariant::custom(c),
            }) => {
                if c.module_index > 0xff || c.method_index > 0xff {
                    return Err(EncodeError::InvalidCallIndex);
                }
                Some((c.module_index as u8, c.method_index as u8))
            },
            _ => None,
        };
        Ok(Self(call_index))
    }

    pub fn required_from_tw(call_index: &Option<CallIndices>) -> Result<Self, EncodeError> {
        if call_index.is_none() {
            return Err(EncodeError::MissingCallIndicesTable);
        }
        Self::from_tw(call_index)
    }

    pub fn has_call_index(&self) -> bool {
        self.0.is_some()
    }

    pub fn wrap<T: ToScale>(self, value: T) -> WithCallIndex<T> {
        WithCallIndex {
            value,
            call_index: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WithCallIndex<T: ToScale> {
    value: T,
    call_index: CallIndex,
}

impl<T: ToScale> WithCallIndex<T> {
    pub fn map<U: ToScale, F: Fn(T) -> U>(self, f: F) -> WithCallIndex<U> {
        WithCallIndex {
            value: f(self.value),
            call_index: self.call_index,
        }
    }
}

impl<T: ToScale> ToScale for WithCallIndex<T> {
    fn to_scale_into(&self, out: &mut Vec<u8>) {
        if let Some(call_index) = &self.call_index.0 {
            let mut value = self.value.to_scale();
            assert!(value.len() >= 2);
            // Override the first two bytes with the custom call index.
            value[0] = call_index.0;
            value[1] = call_index.1;
            out.extend(&value);
        } else {
            self.value.to_scale_into(out);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Encoded(pub Vec<u8>);

impl ToScale for Encoded {
    fn to_scale_into(&self, out: &mut Vec<u8>) {
        out.extend(&self.0);
    }
}

pub struct SignedPayload<'a>((&'a Encoded, &'a Extra, AdditionalSigned));

impl<'a> ToScale for SignedPayload<'a> {
    fn to_scale_into(&self, out: &mut Vec<u8>) {
        self.0.to_scale_into(out);
    }
}

impl<'a> SignedPayload<'a> {
    pub fn new(call: &'a Encoded, extra: &'a Extra, additional: AdditionalSigned) -> Self {
        Self((call, extra, additional))
    }
}

/// PreparedTransaction holds all data needed to sign a transaction.
#[derive(Clone, Debug)]
pub struct PreparedTransaction {
    pub call: Encoded,
    pub extra: Extra,
    pub additional: AdditionalSigned,
}

impl PreparedTransaction {
    pub fn new(additional: AdditionalSigned, extra: Extra, call: Encoded) -> Self {
        Self {
            additional,
            extra,
            call,
        }
    }

    /*
    pub async fn sign(self, signer: &mut impl Signer) -> Result<ExtrinsicV4> {
      let account = signer.account();
      let payload = SignedPayload::new(&self.call, &self.extra, self.additional);
      let payload = payload.to_scale();
      let sig = signer.sign(&payload[..]).await?;

      let xt = ExtrinsicV4::signed(account, sig, self.extra, self.call);
      Ok(xt)
    }
    */
}

impl_struct_scale!(
    #[derive(Clone, Debug)]
    pub struct ExtrinsicSignature {
        pub account: MultiAddress,
        pub signature: MultiSignature,
        pub extra: Extra,
    }
);

/// Current version of the `UncheckedExtrinsic` format.
pub const EXTRINSIC_VERSION: u8 = 4;
pub const SIGNED_EXTRINSIC_BIT: u8 = 0b1000_0000;
pub const UNSIGNED_EXTRINSIC_MASK: u8 = 0b0111_1111;

#[derive(Clone, Debug)]
pub struct ExtrinsicV4 {
    pub signature: Option<ExtrinsicSignature>,
    pub call: Encoded,
}

impl ExtrinsicV4 {
    pub fn signed(account: AccountId, sig: MultiSignature, extra: Extra, call: Encoded) -> Self {
        Self {
            signature: Some(ExtrinsicSignature {
                account: MultiAddress::from(account),
                signature: sig,
                extra,
            }),
            call,
        }
    }
}

impl ToScale for ExtrinsicV4 {
    fn to_scale_into(&self, out: &mut Vec<u8>) {
        // 1 byte version id and signature if signed.
        match &self.signature {
            Some(sig) => {
                out.push(EXTRINSIC_VERSION | SIGNED_EXTRINSIC_BIT);
                sig.to_scale_into(out);
            },
            None => {
                out.push(EXTRINSIC_VERSION & UNSIGNED_EXTRINSIC_MASK);
            },
        }
        self.call.to_scale_into(out);
    }
}
