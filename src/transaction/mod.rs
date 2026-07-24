#![allow(dead_code)]

use std::collections::HashMap;

use crate::error::{Result, TywindbError};
use crate::storage::wal::WalOp;

/// Transaction state
#[derive(Debug, Clone, PartialEq)]
pub enum TxState {
    Active,
    Committed,
    Aborted,
}

/// Transaction
pub struct Transaction {
    pub id: u64,
    pub state: TxState,
    pub writes: Vec<WalOp>,
}

impl Transaction {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            state: TxState::Active,
            writes: Vec::new(),
        }
    }
}

/// Transaction manager
pub struct TransactionManager {
    next_tx_id: u64,
    active_txs: HashMap<u64, Transaction>,
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            next_tx_id: 1,
            active_txs: HashMap::new(),
        }
    }

    pub fn begin(&mut self) -> u64 {
        let tx_id = self.next_tx_id;
        self.next_tx_id += 1;

        let tx = Transaction::new(tx_id);
        self.active_txs.insert(tx_id, tx);

        tx_id
    }

    pub fn get_transaction(&self, tx_id: u64) -> Result<&Transaction> {
        self.active_txs
            .get(&tx_id)
            .ok_or_else(|| TywindbError::Transaction(format!("Transaction {} not found", tx_id)))
    }

    pub fn get_transaction_mut(&mut self, tx_id: u64) -> Result<&mut Transaction> {
        self.active_txs
            .get_mut(&tx_id)
            .ok_or_else(|| TywindbError::Transaction(format!("Transaction {} not found", tx_id)))
    }

    pub fn commit(&mut self, tx_id: u64) -> Result<()> {
        let tx = self.get_transaction_mut(tx_id)?;
        if tx.state != TxState::Active {
            return Err(TywindbError::Transaction(format!(
                "Transaction {} is not active",
                tx_id
            )));
        }
        tx.state = TxState::Committed;
        self.active_txs.remove(&tx_id);
        Ok(())
    }

    pub fn abort(&mut self, tx_id: u64) -> Result<()> {
        let tx = self.get_transaction_mut(tx_id)?;
        if tx.state != TxState::Active {
            return Err(TywindbError::Transaction(format!(
                "Transaction {} is not active",
                tx_id
            )));
        }
        tx.state = TxState::Aborted;
        self.active_txs.remove(&tx_id);
        Ok(())
    }

    pub fn add_write(&mut self, tx_id: u64, operation: WalOp) -> Result<()> {
        let tx = self.get_transaction_mut(tx_id)?;
        tx.writes.push(operation);
        Ok(())
    }
}
