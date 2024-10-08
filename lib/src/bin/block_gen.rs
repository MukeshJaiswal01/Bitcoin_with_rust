use lib::crypto::PrivateKey;
use lib::sha256::Hash;
use lib::types:: {
    Block, BlockHeader, Transaction, TransactionOutput,
};

use lib::util::{MerkleRoot, Saveable};
use chrono::Utc;
use uuid::Uuid;
use std::env;
use std::process::exit;





fn main() {

    let path = if let Some(arg) = env::args().nth(1){
       
        arg

    } else {

        eprintln!("Usage: block_gen <block_file>");

        // code of 1 typically indicates that the program encountered an error or terminated unsuccessfully.
        exit(1);

    };

    let private_key = PrivateKey::new_key();

    let transaction = vec![Transaction::new(
        
        vec![],
        vec![TransactionOutput {
            unique_id: Uuid::new_v4(),
            value: lib::INITIAL_REWARD * 10u64.pow(8),
            pubkey: private_key.public_key(),
        }], 

    )];

    let merkle_root = MerkleRoot::calculate(&transaction);

    let block = Block::new(
        BlockHeader::new(
            Utc::now(),
            0,
            Hash::zero(),
            merkle_root,
            lib::MIN_TARGET,

        ),
        transaction
    );

    block.save_to_file(path).expect("failed to save  block ")




}


