pub use monacoin::{util::address, Block, BlockHeader, OutPoint, Transaction, TxIn, TxOut};

use monacoin::blockdata::constants::genesis_block;
use monacoin::network::constants::Network as BNetwork;
use monacoin::BlockHash;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type Value = u64;

lazy_static! {
    static ref CACHED_GENESIS: Arc<RwLock<HashMap<Network, BlockHash>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Serialize, Ord, PartialOrd, Eq)]
pub enum Network {
    Monacoin,
    MonacoinTestnet,
    MonacoinRegtest,
}

impl Network {
    pub fn genesis_hash(self) -> BlockHash {
        if let Some(block_hash) = CACHED_GENESIS.read().unwrap().get(&self) {
            return *block_hash;
        }

        let block_hash = genesis_block(BNetwork::from(self)).block_hash();
        CACHED_GENESIS.write().unwrap().insert(self, block_hash);
        block_hash
    }

    pub fn magic(self) -> u32 {
        match self {
            Network::Monacoin => 0xD9B4_BEF9,
            Network::MonacoinTestnet => 0x0709_110B,
            Network::MonacoinRegtest => 0xDAB5_BFFA,
        }
    }

    pub fn names() -> Vec<String> {
        return vec![
            "mainnet".to_string(),
            "testnet".to_string(),
            "regtest".to_string(),
        ];
    }
}

impl From<&str> for Network {
    fn from(network_name: &str) -> Self {
        match network_name {
            "mainnet" => Network::Monacoin,
            "testnet" => Network::MonacoinTestnet,
            "regtest" => Network::MonacoinRegtest,

            _ => panic!("unsupported Monacoin network: {:?}", network_name),
        }
    }
}

impl From<Network> for BNetwork {
    fn from(network: Network) -> Self {
        match network {
            Network::Monacoin => BNetwork::Monacoin,
            Network::MonacoinTestnet => BNetwork::MonacoinTestnet,
            Network::MonacoinRegtest => BNetwork::MonacoinRegtest,
        }
    }
}

impl From<BNetwork> for Network {
    fn from(network: BNetwork) -> Self {
        match network {
            BNetwork::Monacoin => Network::Monacoin,
            BNetwork::MonacoinRegtest => Network::MonacoinRegtest,
            BNetwork::MonacoinTestnet => Network::MonacoinTestnet, // @FIXME
        }
    }
}
