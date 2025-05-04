use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub const DIFFICULTY: u32 = 25;

#[derive(Default)]
pub struct BlockHasher {
    id: u64,
}

impl std::hash::Hasher for BlockHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.id
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.id = id;
    }

    #[inline]
    fn write(&mut self, _: &[u8]) {
        // The id will always be a u64, so write_u64 will always be used, and this will never be
        // used.  we need this definition due to the trait.
        // @Student you're not expected to touch this!
        unimplemented!()
    }
}

pub type BlockIdHasher = std::hash::BuildHasherDefault<BlockHasher>;
pub type BlockHashSet = HashSet<u64, BlockIdHasher>;

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct Block {
    /// Hash of the parent block
    pub parent_hash: Vec<u8>,
    /// Miner's (unique) identity. We don't use asymmetric cryptography for this simple exercise.
    pub miner: String,
    /// Random value such the hash value of this block is valid.
    pub nonce: u64,
    /// Dancemove chosen by the miner. That's the very strong incentive explaining
    /// why everyone one wants to mine on this blockchain.
    pub dancemove: DanceMove,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum DanceMove {
    #[default]
    Y = 1,
    M = 2,
    C = 3,
    A = 4,
}

impl Block {
    pub fn new(parent_hash: Vec<u8>, miner: String, nonce: u64, dancemove: DanceMove) -> Self {
        Block {
            parent_hash,
            miner,
            nonce,
            dancemove,
        }
    }

    /// Computes the hash of self
    pub fn hash_block(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // Adding the block's data to the hasher
        hasher.update(&self.parent_hash);
        hasher.update(self.miner.as_bytes());
        hasher.update(&self.nonce.to_be_bytes());
        hasher.update(&(self.dancemove as u8).to_be_bytes());
        
        // hash result
        let result = hasher.finalize();
        
        // Converting to fixed-size array
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Solves the block finding a nonce that hashes the block to
    /// a hash value starting with `difficulty` bits set to 0. Returns the
    /// hash value of the block stored in a Vec.
    pub fn solve_block<R: RngCore>(
        &mut self,
        rng: &mut R,
        difficulty: u32,
        max_iteration: Option<u64>,
    ) -> Option<Vec<u8>> {
        let mut iterations = 0;
        let max_iter = max_iteration.unwrap_or(u64::MAX);
        
        while iterations < max_iter {
            // generating random nonce
            self.nonce = rand::Rng::gen(rng);
            
            // Hashing the block with the current nonce
            let hash = self.hash_block();
            
            // Checking if the hash meets the difficulty requirement
            if self.pow_check(&hash, difficulty) {
                return Some(hash.to_vec());
            }
            
            iterations += 1;
        }
        
        None
    }

    /// Checks if the proof of work is correct
    pub fn pow_check(&self, hash: &[u8], difficulty: u32) -> bool {
        if difficulty == 0 {
            return true;
        }
        
        let full_bytes = difficulty / 8;
        let remaining_bits = difficulty % 8;
        
        // Checking if the required number of full bytes are zero
        for i in 0..full_bytes as usize {
            if i >= hash.len() || hash[i] != 0 {
                return false;
            }
        }
        
        // Checking the remaining bits in the next byte
        if remaining_bits > 0 {
            let byte_pos = full_bytes as usize;
            if byte_pos >= hash.len() {
                return false;
            }
            
            // Creating a mask for the remaining bits (e.g., for 3 bits: 0b11100000)
            let mask = 0xFF << (8 - remaining_bits);
            
            // Checking if the masked bits are all zero
            if (hash[byte_pos] & mask) != 0 {
                return false;
            }
        }
        
        true
    }

    #[allow(dead_code)]
    pub fn is_block_valid(&self, difficulty: u32) -> Result<(), &'static str> {
        // Checking if miner name is valid
        if self.miner == "changemeyoufool" || (self.miner == "Genesis" && !self.parent_hash.is_empty()) {
            return Err("Invalid miner name");
        }
        
        // Checking if dancemove is valid (1-4)
        let dance_value = self.dancemove as u8;
        if dance_value < 1 || dance_value > 4 {
            return Err("Invalid dance move");
        }
        
        // Checking proof of work
        let hash = self.hash_block();
        if !self.pow_check(&hash, difficulty) {
            return Err("Invalid proof of work");
        }
        
        Ok(())
    }

    pub fn is_genesis(&self, difficulty: u32) -> bool {
        if self.parent_hash.is_empty() && self.miner == "Genesis" {
            // checking if the hash of the genesis block is valid
            // and meets the difficulty requirement
            let hash = self.hash_block();
            return self.pow_check(&hash, difficulty);
        }
        false
    }
}

impl crate::simpletree::Parenting for Block {
    fn is_parent(&self, parent_id: &[u8]) -> bool {
        // Check if this block's hash matches the parent_id (which is the parent hash of another block)
        let self_hash = self.hash_block();
        if parent_id.len() != self_hash.len() {
            return false;
        }
        
        // Compare each byte of this block's hash with the parent_id
        for (i, &byte) in self_hash.iter().enumerate() {
            if byte != parent_id[i] {
                return false;
            }
        }
        
        true
    }
    
    fn parent_hash(&self) -> &[u8] {
        &self.parent_hash
    }
    
    fn hash(&self) -> Vec<u8> {
        self.hash_block().to_vec()
    }
    
    fn nonce(&self) -> u64 {
        self.nonce
    }
}

// Remove the RngExt trait entirely as it's causing conflicts with the standard Rng trait

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::Rng;
    use rand::SeedableRng;

    #[test]
    fn test_pow_check() {
        let block = Block {
            parent_hash: vec![],
            miner: "test".to_string(),
            nonce: 0,
            dancemove: DanceMove::C,
        };

        // Test case where hash has sufficient leading zeros
        let hash_with_zeros = vec![0x00, 0x00, 0x00, 0xFF];
        assert!(block.pow_check(&hash_with_zeros, 24));

        // Test case with insufficient zeros
        let hash_without_zeros = vec![0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!block.pow_check(&hash_without_zeros, 1));

        // Test edge case (difficulty = 0)
        assert!(block.pow_check(&hash_without_zeros, 0));
    }

    #[test]
    fn test_solve_block() {
        let mut block = Block {
            parent_hash: vec![],
            miner: "test".to_string(),
            nonce: 0,
            dancemove: DanceMove::Y,
        };
        // Notes for students:
        // Use a seeded Rng for deterministic testing
        // Remember from class 04; a PRG is deterministic and
        // produces the same sequence as long as we choose the same
        // seed.
        let mut rng = StdRng::seed_from_u64(42);
        block.nonce = rand::Rng::gen(&mut rng);

        for difficulty in 5..10 {
            if difficulty % 2 == 0 {
                block.dancemove = DanceMove::A;
            } else {
                block.dancemove = DanceMove::M;
            }

            let hash = block.solve_block(&mut rng, difficulty, None).unwrap();

            // Ensure the solved hash meets the difficulty
            assert!(block.pow_check(&hash, difficulty));

            // Ensure nonce changed
            assert_ne!(block.nonce, 0);
        }
    }

    #[test]
    fn test_new_genesis() {
        let mut genesis = Block::new(Vec::new(), "Genesis".to_string(), 42, DanceMove::C);
        let mut rng = StdRng::seed_from_u64(42);
        genesis.nonce = rand::Rng::gen(&mut rng);
        genesis.solve_block(&mut rng, 10, None).unwrap();
        assert!(genesis.is_genesis(10));
    }
}