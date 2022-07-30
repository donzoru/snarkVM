// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

impl<
    N: Network,
    PreviousHashesMap: for<'a> Map<'a, u32, N::BlockHash>,
    HeadersMap: for<'a> Map<'a, u32, Header<N>>,
    TransactionsMap: for<'a> Map<'a, u32, Transactions<N>>,
    ProgramsMap: for<'a> Map<'a, ProgramID<N>, Deployment<N>>,
> Ledger<N, PreviousHashesMap, HeadersMap, TransactionsMap, ProgramsMap>
{
    /// Returns the block for the given block height.
    pub fn get_block(&self, height: u32) -> Result<Block<N>> {
        Block::from(self.get_previous_hash(height)?, *self.get_header(height)?, self.get_transactions(height)?.clone())
    }

    /// Returns the block hash for the given block height.
    pub fn get_hash(&self, height: u32) -> Result<N::BlockHash> {
        match height.cmp(&self.current_height) {
            Ordering::Equal => Ok(self.current_hash),
            Ordering::Less => match self.previous_hashes.get(&(height + 1))? {
                Some(block_hash) => Ok(*block_hash),
                None => bail!("Missing block hash for block {height}"),
            },
            Ordering::Greater => bail!("Block {height} (given) is greater than the current height"),
        }
    }

    /// Returns the previous block hash for the given block height.
    pub fn get_previous_hash(&self, height: u32) -> Result<N::BlockHash> {
        match self.previous_hashes.get(&height)? {
            Some(previous_hash) => Ok(*previous_hash),
            None => bail!("Missing previous block hash for block {height}"),
        }
    }

    /// Returns the block header for the given block height.
    pub fn get_header(&self, height: u32) -> Result<&Header<N>> {
        match self.headers.get(&height)? {
            Some(header) => Ok(header),
            None => bail!("Missing block header for block {height}"),
        }
    }

    /// Returns the block transactions for the given block height.
    pub fn get_transactions(&self, height: u32) -> Result<&Transactions<N>> {
        match self.transactions.get(&height)? {
            Some(transactions) => Ok(transactions),
            None => bail!("Missing block transactions for block {height}"),
        }
    }

    /// Returns the output records that belong to the given view key.
    pub fn get_output_records<'a>(
        &'a self,
        view_key: &'a ViewKey<N>,
    ) -> impl '_ + Iterator<Item = (Field<N>, Result<Record<N, Plaintext<N>>>)> {
        // Derive the address from the view key.
        let address = view_key.to_address();

        self.transitions().flat_map(Transition::output_records).flat_map(move |(commitment, (record, nonce))| {
            match record.is_owner(&address, view_key, nonce) {
                true => Some((*commitment, record.decrypt(view_key, nonce))),
                false => None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::test_helpers::CurrentLedger;

    #[test]
    fn test_get_block() {
        // Load the genesis block.
        let genesis = Block::from_bytes_le(GenesisBytes::load_bytes()).unwrap();

        // Initialize a new ledger.
        let ledger = CurrentLedger::new().unwrap();
        // Retrieve the genesis block.
        let candidate = ledger.get_block(0).unwrap();
        // Ensure the genesis block matches.
        assert_eq!(genesis, candidate);
    }
}
