use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Event {
  InscriptionCreated {
    block_height: u32,
    charms: u16,
    inscription_id: InscriptionId,
    location: Option<SatPoint>,
    parent_inscription_ids: Vec<InscriptionId>,
    sequence_number: u32,
    block_hash: BlockHash,
  },
  InscriptionTransferred {
    block_height: u32,
    inscription_id: InscriptionId,
    new_location: SatPoint,
    old_location: SatPoint,
    sequence_number: u32,
    block_hash: BlockHash,
  },
  RuneBurned {
    amount: u128,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
    block_hash: BlockHash,
  },
  RuneEtched {
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
    block_hash: BlockHash,
  },
  RuneMinted {
    amount: u128,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
    block_hash: BlockHash,
  },
  RuneTransferred {
    amount: u128,
    block_height: u32,
    outpoint: OutPoint,
    rune_id: RuneId,
    txid: Txid,
    block_hash: BlockHash,
  },
}
