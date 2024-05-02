use async_trait::async_trait;
use alloc::boxed::Box;

use miden_objects::{
    accounts::AccountId, assembly::ModuleAst, notes::NoteId, transaction::TransactionInputs,
};

use crate::DataStoreError;

// DATA STORE TRAIT
// ================================================================================================

/// The [DataStore] trait defines the interface that transaction objects use to fetch data
/// required for transaction execution.
#[async_trait(?Send)]
pub trait DataStore {
    /// Returns account, chain, and input note data required to execute a transaction against
    /// the account with the specified ID and consuming the set of specified input notes.
    ///
    /// block_ref must be the block number of the block by which all of the input notes have been
    /// recorded in the chain. In general, it is recommended that bock_ref corresponds to the
    /// latest block available in the data store.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The account with the specified ID could not be found in the data store.
    /// - The block with the specified number could not be found in the data store.
    /// - Any of the notes with the specified IDs could not be found in the data store.
    /// - Any of the notes with the specified IDs were already consumed.
    /// - The combination of specified inputs resulted in a transaction input error.
    /// - The data store encountered some internal error
    async fn get_transaction_inputs(
        &self,
        account_id: AccountId,
        block_ref: u32,
        notes: &[NoteId],
    ) -> Result<TransactionInputs, DataStoreError>;

    /// Returns the account code [ModuleAst] associated with the specified [AccountId].
    async fn get_account_code(&self, account_id: AccountId) -> Result<ModuleAst, DataStoreError>;
}
