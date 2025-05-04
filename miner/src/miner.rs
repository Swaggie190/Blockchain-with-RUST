use block::Block;
use block::BlockHashSet;
use block::DanceMove;
use block::DIFFICULTY;
use clap::{Parser, Subcommand};
use network::NetworkConnector;
use rand::RngCore;
use simpletree::TreeNode;
use std::fmt;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;
use rand::thread_rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

const MY_NAME: &str = "your_miner_name"; // Change this to your unique miner name

#[derive(Default, Debug)]
struct Blockchain {
    /// The blockchain is represented as a simple tree with no
    /// parent pointer.
    blocks: TreeNode<Block>,
}

impl Blockchain {
    pub fn new_from_genesis(genesis: Block) -> Self {
        let mut blockchain = Blockchain::default();
        blockchain.blocks = TreeNode::new(genesis);
        blockchain
    }

    /// Creates a new Blockchain from the provided genesis
    /// block and vector of valid blocks.
    /// Creates a new Blockchain from the provided genesis
/// block and vector of valid blocks.
    /// Creates a new Blockchain from the provided genesis
/// block and vector of valid blocks.
    pub fn new_from_genesis_and_vec(
        genesis: Block,
        blocks: Vec<Block>,
    ) -> (Self, Vec<Block>) {
        let mut blockchain = Self::new_from_genesis(genesis);
        let mut remaining_blocks = Vec::new();
        let mut processed_blocks = BlockHashSet::default();
        
        // Process blocks and add them to the tree
        let mut blocks_to_process: Vec<Block> = blocks;
        
        // Keep trying to add blocks until we can't add any more
        let mut progress_made = true;
        while progress_made && !blocks_to_process.is_empty() {
            progress_made = false;
            let mut still_to_process = Vec::new();
            
            for block in blocks_to_process {
                // Skip blocks that have already been processed
                if processed_blocks.contains(&block.nonce) {
                    continue;
                }
                
                // Check if this block connects to our tree
                if let Some(parent_node) = blockchain.blocks.look_for_parent(&block.parent_hash) {
                    // Add the block as a child of its parent
                    parent_node.insert(block.clone());
                    // Mark this block as processed
                    processed_blocks.insert(block.nonce);
                    progress_made = true;
                } else {
                    // Keep track of blocks we couldn't add yet
                    still_to_process.push(block);
                }
            }
            
            blocks_to_process = still_to_process;
        }
        
        // Any blocks remaining in blocks_to_process are orphaned
        remaining_blocks = blocks_to_process;
        
        (blockchain, remaining_blocks)
    }
    
    /// Get all chains from the blockchain, from the genesis to each leaf
    pub fn get_chains(&self) -> Vec<Vec<Block>> {
        fn collect_chains(node: &TreeNode<Block>, current_chain: Vec<Block>, chains: &mut Vec<Vec<Block>>) {
            let mut new_chain = current_chain.clone();
            new_chain.push(node.value().clone());
            
            if node.children().is_empty() {
                // If this is a leaf node, add the chain to our collection
                chains.push(new_chain);
            } else {
                // Otherwise, continue recursively for each child
                for child in node.children() {
                    collect_chains(child, new_chain.clone(), chains);
                }
            }
        }
        
        let mut chains = Vec::new();
        collect_chains(&self.blocks, Vec::new(), &mut chains);
        chains
    }
    
    /// Get the longest chain in the blockchain
    pub fn get_longest_chain(&self) -> Vec<Block> {
        let chains = self.get_chains();
        chains.into_iter().max_by_key(|chain| chain.len()).unwrap_or_default()
    }

    fn print_tree(
        &self,
        f: &mut fmt::Formatter<'_>,
        node: &TreeNode<Block>,
        prefixes: &mut Vec<bool>,
    ) -> fmt::Result {
        // Print the current node
        if !prefixes.is_empty() {
            // Print connecting lines from parent
            for &is_last in &prefixes[..prefixes.len() - 1] {
                write!(f, "{}", if is_last { "    " } else { "│   " })?;
            }

            // Print the appropriate connector
            let is_last = *prefixes.last().unwrap();
            write!(f, "{}", if is_last { "└── " } else { "├── " })?;
        }

        // Print the block info
        let block = node.value();
        writeln!(f, "{} (nonce: {})", block.miner, block.nonce)?;

        // Recursively print children
        let child_count = node.children().len();
        for (i, child) in node.children().iter().enumerate() {
            prefixes.push(i == child_count - 1); // true if this is the last child
            self.print_tree(f, child, prefixes)?;
            prefixes.pop();
        }

        Ok(())
    }
}

impl fmt::Display for Blockchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print_tree(f, &self.blocks, &mut Vec::new())
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    action: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Mine {
        #[arg(short, default_value_t = DIFFICULTY)]
        difficulty: u32,
        #[arg(short, default_value_t = String::from(MY_NAME))]
        miner_name: String,
        #[arg(long)]
        max_iter: Option<u64>,
    },
    Print {
        #[arg(short, default_value_t = DIFFICULTY)]
        difficulty: u32,
    },
}

fn mine() {
    // use message passing to communicate between the thread querying the server
    // and sending any new block as a vector of blocks
    let (tx1, rx1) = mpsc::sync_channel(1);
    // use message passing to communicate between the thread(s) mining blocks
    // and the thread interacting with the server.
    let (tx2, rx2) = mpsc::channel();

    thread::spawn(move || {
        let mut net = NetworkConnector::new(tx1, rx2);
        net.sync().expect("Network failure");
    });
    
    // Main mining loop
    let mut blockchain = None;
    let mut rng = thread_rng();
    let difficulty = DIFFICULTY;
    let miner_name = MY_NAME.to_string();
    
    println!("Starting mining with miner name: {}", miner_name);
    println!("Difficulty: {}", difficulty);
    
    loop {
        // Try to receive blocks from the network
        match rx1.try_recv() {
            Ok(new_blocks) => {
                println!("Received {} blocks from network", new_blocks.len());
                
                // Find a genesis block
                if blockchain.is_none() {
                    // Try to find a genesis block
                    for block in &new_blocks {
                        if block.is_genesis(difficulty) {
                            println!("Found genesis block from: {}", block.miner);
                            blockchain = Some(Blockchain::new_from_genesis(block.clone()));
                            break;
                        }
                    }
                    
                    // If we still don't have a genesis block, create one
                    if blockchain.is_none() {
                        println!("Creating own genesis block...");
                        let mut genesis = Block::new(Vec::new(), "Genesis".to_string(), 0, DanceMove::Y);
                        if let Some(hash) = genesis.solve_block(&mut rng, difficulty, None) {
                            println!("Created genesis block with hash: {:?}", hash);
                            blockchain = Some(Blockchain::new_from_genesis(genesis.clone()));
                            // Send the genesis block to network
                            tx2.send(genesis).expect("Failed to send genesis block");
                        }
                    }
                }
                
                // If we have a blockchain, update it with the new blocks
                if let Some(ref mut bc) = blockchain {
                    let (updated_bc, _) = Blockchain::new_from_genesis_and_vec(
                        bc.blocks.value().clone(),
                        new_blocks
                    );
                    *bc = updated_bc;
                }
            },
            Err(TryRecvError::Empty) => {
                // No new blocks, continue
            },
            Err(TryRecvError::Disconnected) => {
                println!("Network connection lost!");
                break;
            }
        }
        
        // If we have a blockchain, try to mine a new block
        if let Some(ref bc) = blockchain {
            // Choose a random dance move
            let dance_moves = [DanceMove::Y, DanceMove::M, DanceMove::C, DanceMove::A];
            let dancemove = dance_moves[rng.next_u32() as usize % dance_moves.len()];
            
            // Get the latest block hash from the longest chain
            let longest_chain = bc.get_longest_chain();
            if let Some(last_block) = longest_chain.last() {
                let parent_hash = last_block.hash_block().to_vec();
                
                // Create and solve a new block
                let mut new_block = Block::new(parent_hash, miner_name.clone(), 0, dancemove);
                if let Some(hash) = new_block.solve_block(&mut rng, difficulty, Some(1000)) {
                    println!("Mined new block with dance move: {:?}, hash: {:?}", dancemove, hash);
                    
                    // Send the new block to the network
                    tx2.send(new_block).expect("Failed to send block");
                }
            }
        }
        
        // Sleep briefly to avoid hogging the CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn print_blockchain(difficulty: u32) {
    // Get all blocks from the server
    match network::get_blocks() {
        Ok(blocks) => {
            // Find genesis blocks
            let mut genesis_blocks = Vec::new();
            for block in &blocks {
                if block.is_genesis(difficulty) {
                    genesis_blocks.push(block.clone());
                }
            }
            
            if genesis_blocks.is_empty() {
                println!("No genesis blocks found");
                return;
            }
            
            // Create a blockchain from each genesis block
            for genesis in genesis_blocks {
                let (blockchain, remaining) = Blockchain::new_from_genesis_and_vec(
                    genesis.clone(),
                    blocks.clone()
                );
                
                println!("Blockchain with genesis from {}", genesis.miner);
                println!("{}", blockchain);
                println!("Longest chain length: {}", blockchain.get_longest_chain().len());
                println!("Remaining blocks: {}", remaining.len());
                println!("-----------------------------------");
            }
        },
        Err(e) => {
            println!("Failed to get blocks from server: {:?}", e);
        }
    }
}

fn main() {
    let args = Args::parse();

    match &args.action {
        Some(Commands::Mine {
            difficulty,
            miner_name,
            max_iter,
        }) => {
            mine();
        }

        Some(Commands::Print { difficulty }) => {
            print_blockchain(*difficulty);
        }

        None => {
            println!("No command specified. Use --help for usage information.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block(parent_hash: &[u8], nonce_init: u64, miner: &str) -> Block {
        Block::new(
            parent_hash.to_vec(),
            miner.to_string(),
            nonce_init,
            DanceMove::Y,
        )
    }

    #[test]
    fn test_empty_blocks() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(genesis, vec![]);

        assert_eq!(blockchain.blocks.children().len(), 0);
    }

    #[test]
    fn test_single_valid_block() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let mut blockids = BlockHashSet::default();
        blockids.insert(42);
        let (blockchain, _) =
            Blockchain::new_from_genesis_and_vec(genesis, vec![block1]);
        assert_eq!(blockids.len(), 1);

        let root = &blockchain.blocks;
        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].value().miner, "miner1");
    }

    #[test]
    fn test_multiple_levels() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        let block3 = create_test_block(&block1_hash, 44, "miner3");

        let mut blockids = BlockHashSet::default();
        blockids.insert(42);
        blockids.insert(43);
        blockids.insert(44);
        let (blockchain, remaining) = Blockchain::new_from_genesis_and_vec(
            genesis,
            vec![block1, block2, block3],
        );

        assert_eq!(blockids.len(), 3);

        let root = &blockchain.blocks;
        assert_eq!(root.children().len(), 2); // block1 and block2

        // Find block1 in children
        let block1_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner1")
            .unwrap();

        assert_eq!(block1_node.children().len(), 1); // block3
        assert_eq!(block1_node.children()[0].value().miner, "miner3");
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_orphaned_blocks() {
        // Create a genesis block
        let genesis = create_test_block(&[], 0, "Genesis");
        // Create a fake hash that doesn't match any block
        let fake_hash = vec![0xFF; 32]; 

        // Create one valid block that connects to genesis
        let valid_block = create_test_block(&genesis.hash_block().to_vec(), 42, "miner1");
        // Create an orphan block with an invalid parent hash
        let orphan_block = create_test_block(&fake_hash, 43, "miner2");

        // Build the blockchain with genesis and both blocks
        let (blockchain, remaining) = Blockchain::new_from_genesis_and_vec(
            genesis.clone(),
            vec![valid_block.clone(), orphan_block.clone()],
        );

        // Verify the valid block was added to the blockchain
        assert_eq!(blockchain.blocks.children().len(), 1);
        assert_eq!(blockchain.blocks.children()[0].value().nonce, 42);
        assert_eq!(blockchain.blocks.children()[0].value().miner, "miner1");
        
        // Verify the orphan block is in the remaining list
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].nonce, 43);
        assert_eq!(remaining[0].miner, "miner2");
    }

    #[test]
    fn test_duplicate_valid_blocks() {
        // Create a genesis block
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        // Create first block off genesis
        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        // Create another block off genesis
        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        
        // Create a block off block1
        let block3 = create_test_block(&block1_hash, 44, "miner3");

        // Build the blockchain including duplicates of the blocks
        let (blockchain, remaining) = Blockchain::new_from_genesis_and_vec(
            genesis.clone(),
            vec![
                block1.clone(), 
                block2.clone(), 
                block3.clone(),
                // Include duplicate blocks with the same nonces
                block1.clone(),
                block2.clone()
            ],
        );

        // Verify structure of the blockchain
        let root = &blockchain.blocks;
        
        // Should have 2 children from genesis (block1 and block2)
        assert_eq!(root.children().len(), 2);
        
        // Find block1 in the children
        let block1_node = root
            .children()
            .iter()
            .find(|n| n.value().nonce == 42)
            .unwrap();
        
        // Verify block1 has block3 as a child
        assert_eq!(block1_node.children().len(), 1);
        assert_eq!(block1_node.children()[0].value().nonce, 44);
        assert_eq!(block1_node.children()[0].value().miner, "miner3");
        
        // Verify no blocks remain unprocessed
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_complex_structure() {
        let genesis = create_test_block(&[], 0, "Genesis");
        let genesis_hash = genesis.hash_block().to_vec();

        // Create blocks
        let block1 = create_test_block(&genesis_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();

        let block2 = create_test_block(&genesis_hash, 43, "miner2");
        let block2_hash = block2.hash_block().to_vec();

        let block3 = create_test_block(&block1_hash, 44, "miner3");
        let block4 = create_test_block(&block2_hash, 45, "miner4");
        let block5 = create_test_block(&block2_hash, 46, "miner5");

        let (blockchain, _) = Blockchain::new_from_genesis_and_vec(
            genesis,
            vec![block1, block2, block3, block4, block5],
        );

        // Verify structure
        let root = &blockchain.blocks;
        assert_eq!(root.children().len(), 2);

        let block1_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner1")
            .unwrap();
        assert_eq!(block1_node.children().len(), 1);
        assert_eq!(block1_node.children()[0].value().miner, "miner3");

        let block2_node = root
            .children()
            .iter()
            .find(|n| n.value().miner == "miner2")
            .unwrap();
        assert_eq!(block2_node.children().len(), 2);
        assert!(block2_node
            .children()
            .iter()
            .any(|n| n.value().miner == "miner4"));
        assert!(block2_node
            .children()
            .iter()
            .any(|n| n.value().miner == "miner5"));
    }

    #[test]
    fn test_multiple_genesis() {
        // Create a primary genesis block 
        let genesis1 = create_test_block(&[], 0, "Genesis");
        let genesis1_hash = genesis1.hash_block().to_vec();
        
        // Create a secondary genesis block with different nonce
        let genesis2 = create_test_block(&[], 1, "Genesis");
        let genesis2_hash = genesis2.hash_block().to_vec();
    
        // Create blocks that descend from genesis1
        let block1 = create_test_block(&genesis1_hash, 42, "miner1");
        let block1_hash = block1.hash_block().to_vec();
        let block2 = create_test_block(&genesis1_hash, 43, "miner2");
        let block3 = create_test_block(&block1_hash, 44, "miner3");
    
        // Create a block that descends from genesis2
        let block4 = create_test_block(&genesis2_hash, 45, "miner4");
    
        // Build blockchain using genesis1, but include blocks from both genesis chains
        let (blockchain, remaining) = Blockchain::new_from_genesis_and_vec(
            genesis1.clone(),
            vec![block1.clone(), block2.clone(), block3.clone(), block4.clone()],
        );
    
        // Verify correct blocks were added to the tree
        assert_eq!(blockchain.blocks.children().len(), 2); // block1 and block2
        
        // Find block1 in the children and verify its child
        let block1_node = blockchain.blocks.children().iter()
            .find(|n| n.value().nonce == 42)
            .unwrap();
        assert_eq!(block1_node.children().len(), 1);
        assert_eq!(block1_node.children()[0].value().nonce, 44);
        
        // Verify blocks from the other genesis chain are in remaining
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].nonce, 45);
        assert_eq!(remaining[0].miner, "miner4");
    }
}

mod block;
mod network;
mod simpletree;