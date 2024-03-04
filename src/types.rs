use serde::{Deserialize, Serialize};

pub const ADDRESS_LENGTH: usize = 20;
pub const HASH_LENGTH: usize = 32;

#[derive(Serialize, Deserialize)]
pub struct OutPoint {
    // Most significant bit is 1 when it is a deposit 0 when not.
    // Remaining 63 bits represent the sequence number of either the deposit or the transaction.
    pub number: u64,
    pub index: u16,
}

impl OutPoint {
    const DEPOSIT_MASK: u64 = 1u64.rotate_right(1);
    const NUMBER_MASK: u64 = !Self::DEPOSIT_MASK;

    pub fn is_deposit(&self) -> bool {
        self.number & Self::DEPOSIT_MASK != 0
    }

    // Deposit number if it is a deposit, transaction number if it is a regular output.
    pub fn number(&self) -> u64 {
        self.number & Self::NUMBER_MASK
    }
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub address: [u8; ADDRESS_LENGTH],
    pub value: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub inputs: Vec<OutPoint>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    pub fn value_out(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Header {
    pub prev_main_block_hash: [u8; HASH_LENGTH],
    pub prev_side_block_hash: [u8; HASH_LENGTH],
    pub merkle_root: [u8; HASH_LENGTH],
}

impl Header {
    fn validate_transactions(&self, transactions: &[Transaction]) -> bool {
        // TODO: Make this into proper merkle root, not just hash of concatenated hashes.
        let merkle_root: [u8; HASH_LENGTH] = blake3::hash(
            &transactions
                .iter()
                .map(|transaction| transaction.hash())
                .collect::<Vec<_>>()
                .concat(),
        )
        .into();

        self.merkle_root == merkle_root
    }
}

pub trait Hashable
where
    Self: Serialize,
{
    fn hash(&self) -> [u8; HASH_LENGTH] {
        let bytes = bincode::serialize(self).unwrap();
        blake3::hash(&bytes).into()
    }
}

impl<T: Serialize> Hashable for T {}
