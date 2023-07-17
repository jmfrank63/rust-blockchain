
use serde::{Serialize, Deserialize};
use crate::simple::simple_sync;

mod blockchain;
mod simple;
// mod p2p;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gods {
    name: String,
    from: String,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_sync()?;



    Ok(())
}
