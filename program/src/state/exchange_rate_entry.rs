use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
#[derive(BorshDeserialize, BorshSerialize, Default, Clone, Copy, Debug)]
pub struct ExchangeRateEntry {
    pub mint: Pubkey, //mint for this entry
    pub rate: u64,    // Exchange rate into the common currency.
    pub decimals: u8, // Mint decimals.
}

impl ExchangeRateEntry {
    pub fn serialized_size() -> usize {
        Self::default()
            .try_to_vec()
            .expect("seriazlied length: ExchangeRateEntry")
            .len()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn foo() {
        let foo = 1;
        assert_eq!(foo, 1);
    }
}
