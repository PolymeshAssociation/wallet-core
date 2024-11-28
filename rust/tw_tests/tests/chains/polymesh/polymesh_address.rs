// SPDX-License-Identifier: Apache-2.0
//
// Copyright © 2017 Trust Wallet.

use crate::chains::polymesh::{PRIVATE_KEY_1, PUBLIC_KEY_1, PUBLIC_KEY_2, PUBLIC_KEY_HEX_1};
use tw_any_coin::test_utils::address_utils::{
    test_address_derive, test_address_get_data, test_address_invalid, test_address_normalization,
    test_address_valid,
};
use tw_coin_registry::coin_type::CoinType;

#[test]
fn test_polymesh_address_derive() {
    test_address_derive(CoinType::Polymesh, PRIVATE_KEY_1, PUBLIC_KEY_1);
}

#[test]
fn test_polymesh_address_normalization() {
    test_address_normalization(CoinType::Polymesh, PUBLIC_KEY_1, PUBLIC_KEY_1);
}

#[test]
fn test_polymesh_address_is_valid() {
    test_address_valid(CoinType::Polymesh, PUBLIC_KEY_1);
    test_address_valid(CoinType::Polymesh, PUBLIC_KEY_2);
}

#[test]
fn test_polymesh_address_invalid() {
    test_address_invalid(
        CoinType::Polymesh,
        "5HUUBD9nsjaKKUVB3XV87CcjcEDu7sDH2G32NAj6uNqgWp9G",
    );
}

#[test]
fn test_polymesh_address_get_data() {
    test_address_get_data(CoinType::Polymesh, PUBLIC_KEY_1, PUBLIC_KEY_HEX_1);
}
