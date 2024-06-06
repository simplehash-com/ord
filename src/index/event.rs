use super::*;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Event {
  InscriptionCreated {
    block_hash: BlockHash,
    block_height: u32,
    charms: u16,
    inscription_id: InscriptionId,
    location: Option<SatPoint>,
    parent_inscription_ids: Vec<InscriptionId>,
    sequence_number: u32,
  },
  InscriptionTransferred {
    block_hash: BlockHash,
    block_height: u32,
    inscription_id: InscriptionId,
    new_location: SatPoint,
    old_location: SatPoint,
    sequence_number: u32,
  },
  RuneBurned {
    block_hash: BlockHash,
    amount: u128,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
  },
  RuneEtched {
    block_hash: BlockHash,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
  },
  RuneMinted {
    block_hash: BlockHash,
    amount: u128,
    block_height: u32,
    rune_id: RuneId,
    txid: Txid,
  },
  RuneTransferred {
    block_hash: BlockHash,
    amount: u128,
    block_height: u32,
    outpoint: OutPoint,
    rune_id: RuneId,
    txid: Txid,
  },
}
