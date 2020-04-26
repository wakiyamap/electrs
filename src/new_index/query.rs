use rayon::prelude::*;

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::{Duration, Instant};

use crate::chain::{OutPoint, Transaction, TxOut};
use crate::daemon::Daemon;
use crate::errors::*;
use crate::new_index::{ChainQuery, Mempool, ScriptStats, SpendingInput, Utxo};
use crate::util::{is_spendable, BlockId, Bytes, TransactionStatus};

use monacoin::Txid;

const FEE_ESTIMATES_TTL: u64 = 60; // seconds

const CONF_TARGETS: [u16; 28] = [
    1u16, 2u16, 3u16, 4u16, 5u16, 6u16, 7u16, 8u16, 9u16, 10u16, 11u16, 12u16, 13u16, 14u16, 15u16,
    16u16, 17u16, 18u16, 19u16, 20u16, 21u16, 22u16, 23u16, 24u16, 25u16, 144u16, 504u16, 1008u16,
];

pub struct Query {
    chain: Arc<ChainQuery>, // TODO: should be used as read-only
    mempool: Arc<RwLock<Mempool>>,
    daemon: Arc<Daemon>,
    cached_estimates: RwLock<(HashMap<u16, f32>, Option<Instant>)>,
    cached_relayfee: RwLock<Option<f64>>,

    #[cfg(feature = "liquid")]
    pub network: Network,
    #[cfg(feature = "liquid")]
    asset_db: Option<AssetRegistry>,
}

impl Query {
    #[cfg(not(feature = "liquid"))]
    pub fn new(chain: Arc<ChainQuery>, mempool: Arc<RwLock<Mempool>>, daemon: Arc<Daemon>) -> Self {
        Query {
            chain,
            mempool,
            daemon,
            cached_estimates: RwLock::new((HashMap::new(), None)),
            cached_relayfee: RwLock::new(None),
        }
    }

    pub fn chain(&self) -> &ChainQuery {
        &self.chain
    }

    pub fn mempool(&self) -> RwLockReadGuard<Mempool> {
        self.mempool.read().unwrap()
    }

    pub fn broadcast_raw(&self, txhex: &str) -> Result<Txid> {
        let txid = self.daemon.broadcast_raw(txhex)?;
        self.mempool
            .write()
            .unwrap()
            .add_by_txid(&self.daemon, &txid);
        Ok(txid)
    }

    pub fn utxo(&self, scripthash: &[u8]) -> Vec<Utxo> {
        let mut utxos = self.chain.utxo(scripthash);
        let mempool = self.mempool();
        utxos.retain(|utxo| !mempool.has_spend(&OutPoint::from(utxo)));
        utxos.extend(mempool.utxo(scripthash));
        utxos
    }

    pub fn history_txids(&self, scripthash: &[u8], limit: usize) -> Vec<(Txid, Option<BlockId>)> {
        let confirmed_txids = self.chain.history_txids(scripthash, limit);
        let confirmed_len = confirmed_txids.len();
        let confirmed_txids = confirmed_txids.into_iter().map(|(tx, b)| (tx, Some(b)));

        let mempool_txids = self
            .mempool()
            .history_txids(scripthash, limit - confirmed_len)
            .into_iter()
            .map(|tx| (tx, None));

        confirmed_txids.chain(mempool_txids).collect()
    }

    pub fn stats(&self, scripthash: &[u8]) -> (ScriptStats, ScriptStats) {
        (
            self.chain.stats(scripthash),
            self.mempool().stats(scripthash),
        )
    }

    pub fn lookup_txn(&self, txid: &Txid) -> Option<Transaction> {
        self.chain
            .lookup_txn(txid, None)
            .or_else(|| self.mempool().lookup_txn(txid))
    }
    pub fn lookup_raw_txn(&self, txid: &Txid) -> Option<Bytes> {
        self.chain
            .lookup_raw_txn(txid, None)
            .or_else(|| self.mempool().lookup_raw_txn(txid))
    }

    pub fn lookup_txos(&self, outpoints: &BTreeSet<OutPoint>) -> HashMap<OutPoint, TxOut> {
        // the mempool lookup_txos() internally looks up confirmed txos as well
        self.mempool()
            .lookup_txos(outpoints)
            .expect("failed loading txos")
    }

    pub fn lookup_spend(&self, outpoint: &OutPoint) -> Option<SpendingInput> {
        self.chain
            .lookup_spend(outpoint)
            .or_else(|| self.mempool().lookup_spend(outpoint))
    }

    pub fn lookup_tx_spends(&self, tx: Transaction) -> Vec<Option<SpendingInput>> {
        let txid = tx.txid();

        tx.output
            .par_iter()
            .enumerate()
            .map(|(vout, txout)| {
                if is_spendable(txout) {
                    self.lookup_spend(&OutPoint {
                        txid,
                        vout: vout as u32,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_tx_status(&self, txid: &Txid) -> TransactionStatus {
        TransactionStatus::from(self.chain.tx_confirming_block(txid))
    }

    pub fn estimate_fee(&self, conf_target: u16) -> Option<f32> {
        if let (ref cache, Some(cache_time)) = *self.cached_estimates.read().unwrap() {
            if cache_time.elapsed() < Duration::from_secs(FEE_ESTIMATES_TTL) {
                return cache.get(&conf_target).copied();
            }
        }

        self.update_fee_estimates();
        self.cached_estimates
            .read()
            .unwrap()
            .0
            .get(&conf_target)
            .copied()
    }

    pub fn estimate_fee_map(&self) -> HashMap<u16, f32> {
        if let (ref cache, Some(cache_time)) = *self.cached_estimates.read().unwrap() {
            if cache_time.elapsed() < Duration::from_secs(FEE_ESTIMATES_TTL) {
                return cache.clone();
            }
        }

        self.update_fee_estimates();
        self.cached_estimates.read().unwrap().0.clone()
    }

    fn update_fee_estimates(&self) {
        match self.daemon.estimatesmartfee_batch(&CONF_TARGETS) {
            Ok(estimates) => {
                *self.cached_estimates.write().unwrap() = (estimates, Some(Instant::now()));
            }
            Err(err) => {
                warn!("failed estimating feerates: {:?}", err);
            }
        }
    }

    pub fn get_relayfee(&self) -> Result<f64> {
        if let Some(cached) = *self.cached_relayfee.read().unwrap() {
            return Ok(cached);
        }

        let relayfee = self.daemon.get_relayfee()?;
        self.cached_relayfee.write().unwrap().replace(relayfee);
        Ok(relayfee)
    }

    #[cfg(feature = "liquid")]
    pub fn new(
        chain: Arc<ChainQuery>,
        mempool: Arc<RwLock<Mempool>>,
        daemon: Arc<Daemon>,
        network: Network,
        asset_db: Option<AssetRegistry>,
    ) -> Self {
        Query {
            chain,
            mempool,
            daemon,
            network,
            asset_db,
            cached_estimates: RwLock::new((HashMap::new(), None)),
            cached_relayfee: RwLock::new(None),
        }
    }

    #[cfg(feature = "liquid")]
    pub fn lookup_asset(&self, asset_id: &AssetId) -> Result<Option<LiquidAsset>> {
        lookup_asset(&self, self.asset_db.as_ref(), asset_id)
    }
}
