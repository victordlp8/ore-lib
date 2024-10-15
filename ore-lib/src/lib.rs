pub mod args;
pub mod commands;
pub mod cu_limits;
pub mod dynamic_fee;
pub mod error;
pub mod miner;
pub mod open;
pub mod pool;
pub mod send_and_confirm;
pub mod utils;

use lazy_static::lazy_static;
use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

use args::MineArgs;
use error::Error;
use miner::Miner;

lazy_static! {
    static ref GLOBAL_MANAGER: Arc<Mutex<Manager>> = Arc::new(Mutex::new(Manager::default()));
}

pub struct Manager {
    pub miner: Miner,
    pub mining_args: MineArgs,
    pub is_mining: Arc<AtomicBool>,
    pub stop_sender: mpsc::Sender<()>,
}

impl Default for Miner {
    fn default() -> Self {
        Miner::new(
            Arc::new(RpcClient::new(
                "https://api.mainnet-beta.solana.com".to_string(),
            )),
            None,
            None,
            None,
            false,
            None,
            Arc::new(RpcClient::new(
                "https://api.mainnet-beta.solana.com".to_string(),
            )),
            Arc::new(std::sync::RwLock::new(0)),
        )
    }
}

impl Default for MineArgs {
    fn default() -> Self {
        MineArgs {
            pool_url: None,
            cores: 1,
            buffer_time: 0,
            boost_1: None,
            boost_2: None,
            boost_3: None,
        }
    }
}

impl Manager {
    pub fn default() -> Self {
        let (stop_sender, _) = mpsc::channel(1);
        Manager {
            miner: Miner::default(),
            mining_args: MineArgs::default(),
            is_mining: Arc::new(AtomicBool::new(false)),
            stop_sender,
        }
    }

    pub fn new(miner: Miner, mining_args: MineArgs) -> Self {
        let (stop_sender, _) = mpsc::channel(1);
        Manager {
            miner,
            mining_args,
            is_mining: Arc::new(AtomicBool::new(false)),
            stop_sender,
        }
    }

    pub fn set_global_manager(manager: Manager) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut global_manager = GLOBAL_MANAGER.lock().await;
            *global_manager = manager;
        });
    }

    pub fn get_global_manager() -> Arc<Mutex<Manager>> {
        Arc::clone(&GLOBAL_MANAGER)
    }

    pub fn start_mining(&mut self) -> Result<(), Error> {
        if self.is_mining.load(Ordering::SeqCst) {
            return Err(Error::AlreadyMining);
        }

        self.is_mining.store(true, Ordering::SeqCst);
        let is_mining = Arc::clone(&self.is_mining);
        let mining_args = self.mining_args.clone();
        let miner = self.miner.clone();

        tokio::spawn(async move {
            if let Err(e) = miner.mine(mining_args.clone(), &is_mining).await {
                eprintln!("Error during mining: {:?}", e);
            }
        });

        Ok(())
    }

    pub fn stop_mining(&self) -> Result<(), Error> {
        if !self.is_mining.load(Ordering::SeqCst) {
            return Err(Error::NotMining);
        }

        self.is_mining.store(false, Ordering::SeqCst);
        println!("ore-lib: Mining stopped");
        
        Ok(())
    }
}
