#![allow(dead_code)]
use log::{error, info, warn};
use serde::de::{DeserializeOwned, Error as DeError};
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::{
    fmt::Debug,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DIFFICULTY_PREFIX: &str = "00";
pub const GENESIS_HASH: &str = "0000f816a87f806bb0073dcf026a64fb40c946b5abee2573702828694d5b4c43";

pub fn hash_to_binary_representation(hash: &[u8]) -> String {
    let mut res: String = String::default();
    for c in hash {
        res.push_str(&format!("{:08b}", c));
    }
    res
}

pub fn calculate_hash<D>(
    id: u64,
    timestamp: SerializableInstant,
    prev_hash: &str,
    data: &D,
    nonce: u64,
) -> Result<Vec<u8>, Box<dyn Error>>
where
    D: Debug + Clone + Serialize + DeserializeOwned,
{
    let mut hasher = Sha256::default();
    let reference = TimeReference::new();
    let system_time = reference.instant_to_system_time(timestamp.0);
    let duration_since_epoch = system_time.duration_since(UNIX_EPOCH)?;

    // Convert fields to bytes and feed them to the hasher
    hasher.update(id.to_be_bytes());
    hasher.update(duration_since_epoch.as_secs().to_be_bytes());
    hasher.update(prev_hash.as_bytes());

    let serialized_data = serde_json::to_string(data)?;
    hasher.update(serialized_data.as_bytes());

    hasher.update(nonce.to_be_bytes());

    // Read hash digest and consume hasher
    Ok(hasher.finalize().to_vec())
}

fn mine_block<D>(
    id: u64,
    timestamp: SerializableInstant,
    prev_hash: &str,
    data: &D,
) -> Result<(u64, String), Box<dyn Error>>
where
    D: Debug + Clone + Serialize + DeserializeOwned,
{
    info!("mining block...");
    let mut nonce = 0;

    loop {
        if nonce % 100000 == 0 {
            info!("nonce: {}", nonce);
        }
        let hash = calculate_hash(id, timestamp, prev_hash, data, nonce)?;
        let binary_hash = hash_to_binary_representation(&hash);
        if binary_hash.starts_with(DIFFICULTY_PREFIX) {
            info!(
                "mined! nonce: {},\nhash: {}, \nbinary hash: {}",
                nonce,
                hex::encode(&hash),
                binary_hash
            );
            return Ok((nonce, hex::encode(hash)));
        }
        nonce += 1;
    }
}

pub struct TimeReference {
    instant: Instant,
    system_time: SystemTime,
}

impl TimeReference {
    pub fn new() -> Self {
        TimeReference {
            instant: Instant::now(),
            system_time: SystemTime::now(),
        }
    }

    pub fn instant_to_system_time(&self, instant: Instant) -> SystemTime {
        let duration_since_ref = instant.duration_since(self.instant);
        self.system_time + duration_since_ref
    }

    pub fn system_time_to_instant(
        &self,
        system_time: SystemTime,
    ) -> Result<Instant, Box<dyn Error>> {
        let duration_since_ref = system_time.duration_since(self.system_time)?;
        Ok(self.instant + duration_since_ref)
    }
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct SerializableInstant(Instant);

impl From<Instant> for SerializableInstant {
    fn from(instant: Instant) -> Self {
        SerializableInstant(instant)
    }
}

impl Serialize for SerializableInstant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let reference = TimeReference::new();
        let system_time = reference.instant_to_system_time(self.0);
        let duration_since_epoch = system_time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| S::Error::custom("SystemTime before UNIX EPOCH"))?;
        let secs = duration_since_epoch.as_secs();
        serializer.serialize_u64(secs)
    }
}

impl<'de> Deserialize<'de> for SerializableInstant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(secs);
        let reference = TimeReference::new();
        let instant = match reference.system_time_to_instant(system_time) {
            Ok(instant) => instant,
            Err(err) => {
                return Err(D::Error::custom(err.to_string()));
            }
        };
        Ok(SerializableInstant(instant))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
#[serde(bound = "D: Serialize + DeserializeOwned")]
pub struct Block<D>
where
    D: Debug + Clone + Serialize + DeserializeOwned,
{
    pub id: u64,
    pub timestamp: SerializableInstant,
    pub hash: String,
    pub prev_hash: String,
    pub nonce: u64,
    pub data: D,
}

impl<D> Block<D>
where
    D: Debug + Clone + Serialize + DeserializeOwned,
{
    pub fn new(id: u64, prev_hash: String, data: D) -> Result<Self, Box<dyn Error>> {
        let timestamp = SerializableInstant(Instant::now());
        let (nonce, hash) = mine_block(id, timestamp, &prev_hash, &data)?;
        Ok(Self {
            id,
            hash,
            timestamp,
            prev_hash,
            data,
            nonce,
        })
    }
}

pub struct BlockChain<D>
where
    D: Debug + Clone + Serialize + DeserializeOwned,
{
    pub blocks: Vec<Block<D>>,
}

impl<D> BlockChain<D>
where
    D: Debug + Clone + Serialize + DeserializeOwned,
{
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    pub fn genesis(&mut self, data: D) {
        let genesis_block = Block {
            id: 0,
            timestamp: SerializableInstant(Instant::now()),
            prev_hash: String::from("genesis"),
            data,
            nonce: 2836,
            hash: GENESIS_HASH.to_string(),
        };
        self.blocks.push(genesis_block);
    }

    pub fn try_add_block(&mut self, block: Block<D>) -> Result<String, Box<dyn Error>> {
        let latest_block = self
            .blocks
            .last()
            .ok_or("No block found, there must at least be one block")?;
        if self.is_block_valid(&block, latest_block)? {
            let hash = block.hash.clone();
            self.blocks.push(block);
            Ok(hash)
        } else {
            error!("Tried to add an invalid block");
            Err("Tried to add an invalid block".into())
        }
    }

    pub fn is_block_valid(
        &self,
        block: &Block<D>,
        prev_block: &Block<D>,
    ) -> Result<bool, Box<dyn Error>> {
        if block.prev_hash != prev_block.hash {
            warn!("block with id: {} has wrong previous hash", block.id);
            return Ok(false);
        } else if !hash_to_binary_representation(&hex::decode(&block.hash)?)
            .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("block with id: {} has invalid difficulty", block.id);
            return Ok(false);
        } else if block.id != prev_block.id + 1 {
            warn!(
                "block with id: {} is not the next block after the latest: {}",
                block.id, prev_block.id
            );
            return Ok(false);
        } else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.prev_hash,
            &block.data,
            block.nonce,
        )?) != block.hash
        {
            warn!("block with id: {} has invalid hash", block.id);
            return Ok(false);
        }
        Ok(true)
    }

    pub fn is_chain_valid(&self, chain: &[Block<D>]) -> Result<bool, Box<dyn Error>> {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }
            let previous = chain.get(i - 1).ok_or("Previous block has to exist")?;
            let current = chain.get(i).ok_or("Current block has to exist")?;
            if !self.is_block_valid(current, previous)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    // We always choose the longest valid chain
    pub fn choose_chain(
        &mut self,
        local: Vec<Block<D>>,
        remote: Vec<Block<D>>,
    ) -> Result<Vec<Block<D>>, Box<dyn Error>> {
        let is_local_valid = self.is_chain_valid(&local)?;
        let is_remote_valid = self.is_chain_valid(&remote)?;

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                Ok(local)
            } else {
                Ok(remote)
            }
        } else if is_remote_valid && !is_local_valid {
            Ok(remote)
        } else if !is_remote_valid && is_local_valid {
            Ok(local)
        } else {
            error!("local and remote chains are both invalid");
            Err("local and remote chains are both invalid".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, Hash)]
    struct TestData {
        value: String,
    }

    #[test]
    fn test_hash_to_binary_representation() {
        let data = b"genesis";
        let binary = hash_to_binary_representation(data);
        assert_eq!(
            binary,
            "01100111011001010110111001100101011100110110100101110011"
        );
    }

    #[test]
    fn test_calculate_hash() {
        let data = TestData {
            value: "genesis".to_string(),
        };
        let timestamp = SerializableInstant(Instant::now());
        let result = calculate_hash(1, timestamp, "prev_hash", &data, 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mine_block() {
        let data = TestData {
            value: "test".to_string(),
        };
        let timestamp = SerializableInstant(Instant::now());
        let result = mine_block(1, timestamp, "prev_hash", &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_new() {
        let data = TestData {
            value: "test".to_string(),
        };
        let result = Block::new(1, "prev_hash".to_string(), data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_app_try_add_block() {
        let data = TestData {
            value: "test".to_string(),
        };
        let mut app = BlockChain::new();
        app.genesis(data.clone());

        let new_block = Block::new(1, GENESIS_HASH.to_string(), data).unwrap();
        let result = app.try_add_block(new_block);
        assert!(result.is_ok());
        assert_eq!(app.blocks.len(), 2);
    }

    #[test]
    fn test_app_is_block_valid() {
        let data = TestData {
            value: "test".to_string(),
        };
        let mut app = BlockChain::new();
        app.genesis(data.clone());

        let new_block = Block::new(1, GENESIS_HASH.to_string(), data).unwrap();
        let result = app.is_block_valid(&new_block, &app.blocks[0]);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_app_is_chain_valid() {
        let data = TestData {
            value: "test".to_string(),
        };
        let mut app = BlockChain::new();
        app.genesis(data.clone());

        for i in 1..5 {
            let new_block =
                Block::new(i, app.blocks[i as usize - 1].hash.clone(), data.clone()).unwrap();
            app.try_add_block(new_block).unwrap();
        }

        let result = app.is_chain_valid(&app.blocks);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_app_choose_chain() {
        let data = TestData {
            value: "test".to_string(),
        };
        let mut app1 = BlockChain::new();
        app1.genesis(data.clone());
        for i in 1..5 {
            let new_block =
                Block::new(i, app1.blocks[i as usize - 1].hash.clone(), data.clone()).unwrap();
            app1.try_add_block(new_block).unwrap();
        }

        let mut app2 = BlockChain::new();
        app2.genesis(data.clone());
        for i in 1..7 {
            let new_block =
                Block::new(i, app2.blocks[i as usize - 1].hash.clone(), data.clone()).unwrap();
            app2.try_add_block(new_block).unwrap();
        }

        let result = app1.choose_chain(app1.blocks.clone(), app2.blocks.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 7);
    }
}
