use alloc::{string::String, vec::Vec};

use assembly::Assembler;
use miden_crypto::merkle::Smt;
use vm_core::{Felt, FieldElement, Word, ZERO};
use vm_processor::Digest;

use super::{
    account_code::mock_account_code, assets::non_fungible_asset_2,
    constants::FUNGIBLE_FAUCET_INITIAL_BALANCE, prepare_word,
};
use crate::{
    accounts::{
        account_id::testing::{
            ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN, ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN_2,
            ACCOUNT_ID_NON_FUNGIBLE_FAUCET_ON_CHAIN,
            ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN,
            ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN,
            ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN,
        },
        code::testing::make_account_code,
        get_account_seed_single, Account, AccountDelta, AccountId, AccountStorage,
        AccountStorageDelta, AccountStorageType, AccountType, AccountVaultDelta, SlotItem,
        StorageMap, StorageSlot, StorageSlotType,
    },
    assets::{Asset, AssetVault, FungibleAsset},
    notes::NoteAssets,
    testing::account::mock_account,
};

#[derive(Default, Debug, Clone)]
pub struct AccountStorageBuilder {
    items: Vec<SlotItem>,
    maps: Vec<StorageMap>,
}

/// Builder for an `AccountStorage`, the builder can be configured and used multiple times.
impl AccountStorageBuilder {
    pub fn new() -> Self {
        Self { items: vec![], maps: vec![] }
    }

    pub fn add_item(&mut self, item: SlotItem) -> &mut Self {
        self.items.push(item);
        self
    }

    pub fn add_items<I: IntoIterator<Item = SlotItem>>(&mut self, items: I) -> &mut Self {
        for item in items.into_iter() {
            self.add_item(item);
        }
        self
    }

    #[allow(dead_code)]
    pub fn add_map(&mut self, map: StorageMap) -> &mut Self {
        self.maps.push(map);
        self
    }

    #[allow(dead_code)]
    pub fn add_maps<I: IntoIterator<Item = StorageMap>>(&mut self, maps: I) -> &mut Self {
        self.maps.extend(maps);
        self
    }

    pub fn build(&self) -> AccountStorage {
        AccountStorage::new(self.items.clone(), self.maps.clone()).unwrap()
    }
}

// ACCOUNT STORAGE UTILS
// ================================================================================================

pub const FAUCET_STORAGE_DATA_SLOT: u8 = 254;

pub const STORAGE_INDEX_0: u8 = 20;
pub const STORAGE_VALUE_0: Word = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
pub const STORAGE_INDEX_1: u8 = 30;
pub const STORAGE_VALUE_1: Word = [Felt::new(5), Felt::new(6), Felt::new(7), Felt::new(8)];

pub const STORAGE_INDEX_2: u8 = 40;
pub const STORAGE_LEAVES_2: [(Digest, Word); 2] = [
    (
        Digest::new([Felt::new(101), Felt::new(102), Felt::new(103), Felt::new(104)]),
        [Felt::new(1_u64), Felt::new(2_u64), Felt::new(3_u64), Felt::new(4_u64)],
    ),
    (
        Digest::new([Felt::new(105), Felt::new(106), Felt::new(107), Felt::new(108)]),
        [Felt::new(5_u64), Felt::new(6_u64), Felt::new(7_u64), Felt::new(8_u64)],
    ),
];

pub fn storage_item_0() -> SlotItem {
    SlotItem {
        index: STORAGE_INDEX_0,
        slot: StorageSlot::new_value(STORAGE_VALUE_0),
    }
}

pub fn storage_item_1() -> SlotItem {
    SlotItem {
        index: STORAGE_INDEX_1,
        slot: StorageSlot::new_value(STORAGE_VALUE_1),
    }
}

pub fn storage_map_2() -> StorageMap {
    StorageMap::with_entries(STORAGE_LEAVES_2).unwrap()
}

pub fn storage_item_2() -> SlotItem {
    SlotItem {
        index: STORAGE_INDEX_2,
        slot: StorageSlot::new_map(Word::from(storage_map_2().root())),
    }
}

// MOCK FAUCET
// ================================================================================================

pub fn mock_fungible_faucet(
    account_id: u64,
    nonce: Felt,
    empty_reserved_slot: bool,
    assembler: &Assembler,
) -> Account {
    let initial_balance = if empty_reserved_slot {
        ZERO
    } else {
        Felt::new(FUNGIBLE_FAUCET_INITIAL_BALANCE)
    };
    let account_storage = AccountStorage::new(
        vec![SlotItem {
            index: FAUCET_STORAGE_DATA_SLOT,
            slot: StorageSlot::new_value([ZERO, ZERO, ZERO, initial_balance]),
        }],
        vec![],
    )
    .unwrap();
    let account_id = AccountId::try_from(account_id).unwrap();
    let account_code = mock_account_code(assembler);
    Account::from_parts(account_id, AssetVault::default(), account_storage, account_code, nonce)
}

pub fn mock_non_fungible_faucet(
    account_id: u64,
    nonce: Felt,
    empty_reserved_slot: bool,
    assembler: &Assembler,
) -> Account {
    let entries = match empty_reserved_slot {
        true => vec![],
        false => vec![(
            Word::from(non_fungible_asset_2(ACCOUNT_ID_NON_FUNGIBLE_FAUCET_ON_CHAIN)).into(),
            non_fungible_asset_2(ACCOUNT_ID_NON_FUNGIBLE_FAUCET_ON_CHAIN).into(),
        )],
    };

    // construct nft tree
    let nft_tree = Smt::with_entries(entries).unwrap();

    // TODO: add nft tree data to account storage?

    let account_storage = AccountStorage::new(
        vec![SlotItem {
            index: FAUCET_STORAGE_DATA_SLOT,
            slot: StorageSlot::new_map(*nft_tree.root()),
        }],
        vec![],
    )
    .unwrap();
    let account_id = AccountId::try_from(account_id).unwrap();
    let account_code = mock_account_code(assembler);
    Account::from_parts(account_id, AssetVault::default(), account_storage, account_code, nonce)
}

// ACCOUNT SEED GENERATION
// ================================================================================================

pub enum AccountSeedType {
    FungibleFaucetInvalidInitialBalance,
    FungibleFaucetValidInitialBalance,
    NonFungibleFaucetInvalidReservedSlot,
    NonFungibleFaucetValidReservedSlot,
    RegularAccountUpdatableCodeOnChain,
    RegularAccountUpdatableCodeOffChain,
}

/// Returns the account id and seed for the specified account type.
pub fn generate_account_seed(
    account_seed_type: AccountSeedType,
    assembler: &Assembler,
) -> (AccountId, Word) {
    let init_seed: [u8; 32] = Default::default();

    let (account, account_type) = match account_seed_type {
        AccountSeedType::FungibleFaucetInvalidInitialBalance => (
            mock_fungible_faucet(
                ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN,
                ZERO,
                false,
                assembler,
            ),
            AccountType::FungibleFaucet,
        ),
        AccountSeedType::FungibleFaucetValidInitialBalance => (
            mock_fungible_faucet(
                ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN,
                ZERO,
                true,
                assembler,
            ),
            AccountType::FungibleFaucet,
        ),
        AccountSeedType::NonFungibleFaucetInvalidReservedSlot => (
            mock_non_fungible_faucet(
                ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN,
                ZERO,
                false,
                assembler,
            ),
            AccountType::NonFungibleFaucet,
        ),
        AccountSeedType::NonFungibleFaucetValidReservedSlot => (
            mock_non_fungible_faucet(
                ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_ON_CHAIN,
                ZERO,
                true,
                assembler,
            ),
            AccountType::NonFungibleFaucet,
        ),
        AccountSeedType::RegularAccountUpdatableCodeOnChain => (
            mock_account(
                ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN,
                Felt::ONE,
                mock_account_code(assembler),
            ),
            AccountType::RegularAccountUpdatableCode,
        ),
        AccountSeedType::RegularAccountUpdatableCodeOffChain => (
            mock_account(
                ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN,
                Felt::ONE,
                mock_account_code(assembler),
            ),
            AccountType::RegularAccountUpdatableCode,
        ),
    };

    let seed = get_account_seed_single(
        init_seed,
        account_type,
        AccountStorageType::OnChain,
        account.code().root(),
        account.storage().root(),
    )
    .unwrap();

    let account_id = AccountId::new(seed, account.code().root(), account.storage().root()).unwrap();

    (account_id, seed)
}

// UTILITIES
// --------------------------------------------------------------------------------------------

pub fn build_account(assets: Vec<Asset>, nonce: Felt, storage_items: Vec<Word>) -> Account {
    let id = AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_IMMUTABLE_CODE_ON_CHAIN).unwrap();
    let code = make_account_code();

    // build account data
    let vault = AssetVault::new(&assets).unwrap();

    let slot_type = StorageSlotType::Value { value_arity: 0 };
    let slot_items: Vec<SlotItem> = storage_items
        .into_iter()
        .enumerate()
        .map(|(index, item)| SlotItem {
            index: index as u8,
            slot: StorageSlot { slot_type, value: item },
        })
        .collect();
    let storage = AccountStorage::new(slot_items, vec![]).unwrap();

    Account::from_parts(id, vault, storage, code, nonce)
}

pub fn build_account_delta(
    added_assets: Vec<Asset>,
    removed_assets: Vec<Asset>,
    nonce: Felt,
    storage_delta: AccountStorageDelta,
) -> AccountDelta {
    let vault_delta = AccountVaultDelta { added_assets, removed_assets };
    AccountDelta::new(storage_delta, vault_delta, Some(nonce)).unwrap()
}

pub fn build_assets() -> (Asset, Asset) {
    let faucet_id_0 = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).unwrap();
    let asset_0: Asset = FungibleAsset::new(faucet_id_0, 123).unwrap().into();

    let faucet_id_1 = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN_2).unwrap();
    let asset_1: Asset = FungibleAsset::new(faucet_id_1, 345).unwrap().into();

    (asset_0, asset_1)
}

pub fn prepare_assets(note_assets: &NoteAssets) -> Vec<String> {
    let mut assets = Vec::new();
    for &asset in note_assets.iter() {
        let asset_word: Word = asset.into();
        let asset_str = prepare_word(&asset_word);
        assets.push(asset_str);
    }
    assets
}