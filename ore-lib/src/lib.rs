pub mod commands;
pub mod args;
pub mod cu_limits;
pub mod dynamic_fee;
pub mod error;
#[cfg(feature = "admin")]
pub mod initialize;
pub mod open;
pub mod pool;
pub mod send_and_confirm;
pub mod utils;
pub mod miner;

use args::MineArgs;
use error::Error;
use miner::Miner;

pub struct Manager {
    pub miner: Miner,
    pub mining_args: MineArgs,
}

impl Manager {
    pub fn new(miner: Miner, mining_args: MineArgs) -> Self {
        Manager { miner, mining_args }
    }

    pub async fn mine(&self) -> Result<(), Error> {
        self.miner.mine(self.mining_args.clone()).await
    }
}
