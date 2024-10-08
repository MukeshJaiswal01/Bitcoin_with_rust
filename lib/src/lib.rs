use uint ::construct_uint;   // uint : create large fixed size integer ( 256 bit)
use serde::{Serialize, Deserialize};

construct_uint!{

    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}

// initial rewa5rd in bitcoin  - multiply by 10^8 to get satoshis

pub const INITIAL_REWARD: u64 = 50;

// halving interval in blocks 

pub const HALVING_INTERVAL: u64 = 210;

// Ideal block time in seconds  @note actual is average of 10 minutes

pub const IDEAL_BLOCK_TIME: u64 = 10;

// minimum target

pub const MIN_TARGET: U256 = U256([
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFF,
    0x0000_FFFF_FFFF_FFFF,
    ]);

// Difficulty update interval in blocks

pub const  DIFFICULTY_UPDATE_INTERVAL: u64 = 50;

// maximum mempool transaction age in seconds

pub const MAX_MEMPOOL_TRANSACTION_AGE: u64 = 600;



pub mod sha256;
pub mod types;
pub mod util;
pub mod crypto;
pub mod error;
pub mod network;


