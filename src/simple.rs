
use serde::{Deserialize, Serialize};

use crate::blockchain::{BlockChain, Block, GENESIS_HASH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Human {
    name: String,
    age: u32,
}

pub fn simple_sync() -> Result<(), Box<dyn std::error::Error>> {
println!("Simple Blockchain in Rust");
    pretty_env_logger::init();

    let mut human_chain = BlockChain::new();
    let adam = Human {
        name: "Adam".to_string(),
        age: 930,
    };
    human_chain.genesis(adam);

    let seth = Human {
        name: "Seth".to_string(),
        age: 912,
    };

    let seth = Block::new(1, GENESIS_HASH.to_string(), seth)?;
    let seth_hash = human_chain.try_add_block(seth)?;

    let enos = Human {
        name: "Enos".to_string(),
        age: 905,
    };

    let enos = Block::new(2, seth_hash, enos)?;
    let _enos_hash = human_chain.try_add_block(enos)?;

    let lucifer = Human {
        name: "Lucifer".to_string(),
        age: 895,
    };

    let lucifer = Block::new(3, "Vicious but useless try".to_string(), lucifer)?;

    match human_chain.try_add_block(lucifer) {
        Ok(_) => println!("This should never happen!"),
        Err(e) => println!("Expected Error adding Lucifer: {}", e),
    }

    Ok(())
}
