// Experimental code with the p2plib library
#![allow(dead_code)]
use crate::blockchain::{Block, BlockChain};

use libp2p::futures::channel::mpsc;
use libp2p::gossipsub::{self, Sha256Topic as Topic};
use libp2p::mdns::async_io::Behaviour;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{identity, PeerId};
use once_cell::sync::Lazy;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;

pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains".to_string()));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks".to_string()));

#[derive(Debug, Serialize)]
pub struct ChainResponse {
    pub blocks: Vec<Block<String>>,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum Command {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

#[derive(NetworkBehaviour)]
pub struct BlockChainBehaviour
{
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: Behaviour,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<Command>,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender<Command>,
    #[behaviour(ignore)]
    pub block_chain: BlockChain<String>,
}

impl BlockChainBehaviour {
    pub async fn new(
        gossipsub: gossipsub::Behaviour,
        mdns: Behaviour,
        block_chain: BlockChain<String>,
        response_sender: mpsc::UnboundedSender<Command>,
        init_sender: mpsc::UnboundedSender<Command>,
    ) -> Result<Self, Box<dyn Error>> {
        let config = libp2p::mdns::Config::default();
        let mut behaviour = Self {
            block_chain,
            gossipsub,
            mdns,
            response_sender,
            init_sender,
        };
        behaviour.gossipsub.subscribe(&CHAIN_TOPIC.clone());
        behaviour.gossipsub.subscribe(&BLOCK_TOPIC.clone());

        Ok(behaviour)
    }
}
