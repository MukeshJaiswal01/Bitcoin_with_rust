use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{Block, Transaction, TransactionOutput};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::U256;
use std::collections::{HashMap, HashSet};

use crate::util::Saveable;
use std::io::{
Error as IoError, ErrorKind as IoErrorKind, Read,
Result as IoResult, Write,
};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {

    blocks: Vec<Block>,
    target: U256,
    utxos: HashMap<Hash, (bool, TransactionOutput)>,

    #[serde(default, skip_serializing)]
    mempool: Vec<(DateTime<Utc>, Transaction)>,
   
    
}

impl Blockchain {

    pub fn new() -> Self {

        Blockchain { 
            
            blocks: vec![],
            target:crate::MIN_TARGET,
            utxos: HashMap::new(),
            mempool: vec![],

           

        }
    }
    

    // utxos
    pub fn utxos(&self) -> &HashMap<Hash, (bool, TransactionOutput)> {

       &self.utxos
    }


    // target

    pub fn target(&self) -> U256 {

        self.target
    }

    // blocks

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {

        self.blocks.iter()
    }

    // mempool 

    pub fn mempool(&self) -> &[(DateTime<Utc>, Transaction)] {

        &self.mempool
    }


    // add a transaction to mempool


    pub fn add_to_mempool(&mut self, transaction: Transaction) -> Result<()> {

        // validate transaction before insertion
        // all inputs must known Utxos, and must be unique

        let mut known_inputs = HashSet::new();

        for input in &transaction.inputs {

            if !self.utxos.contains_key(&input.prev_transaction_output_hash) {

                return Err(BtcError::InvalidTransaction);
            }

            if known_inputs.contains(&input.prev_transaction_output_hash) {

                return Err(BtcError::InvalidTransaction);
            }

            known_inputs.insert(input.prev_transaction_output_hash);
        }

        // check if any of the utxos have the bool mark set to true 
        // and if so , find the transaction that reference them in mempool
        // remove it and set all the utxos it reference to false
        

        for input in &transaction.inputs{ 

            if let Some((true, _)) = self.utxos.get(&input.prev_transaction_output_hash) { //  do the matching pattern run if the return value is true


                // find the transaction that references the utxo , we are trying to reference

                let referencing_transaction = self.mempool  
                    .iter()
                    .enumerate()
                    .find(
                            |(_, (_, transaction)) | {

                                transaction.outputs.iter()
                                    .any(|output | {

                                        output.hash() == input.prev_transaction_output_hash
                                    })
                            }
                    );

                // if we have found one, unmark all of its UTXOs

                if let Some(( idx, (_, referencing_transaction))) = referencing_transaction {

                    for input in &referencing_transaction.inputs {

                        // set all utxos from this transaction to false


                        self.utxos.entry(input.prev_transaction_output_hash).and_modify(|(marked, _)| {

                            *marked = false;
                        });           
                    }


                    // remove the transaction from the mempool 

                    self.mempool.remove(idx);
                
                } else  {

                    // if some how  there is no matching - set this utxo to false

                    self.utxos.entry(input.prev_transaction_output_hash).and_modify(|(marked, _)| {

                        *marked = false;
                    });
                
                }



            }
        }




        let all_inputs = transaction.inputs
            .iter()
            .map(|input| {
            self.utxos
            .get(
            &input.prev_transaction_output_hash,
            )
            .expect("BUG: impossible")
            .1
            .value
            })
            .sum::<u64>();

        let all_outputs = transaction.outputs
            .iter()
            .map(|output| output.value)
            .sum();
            if all_inputs < all_outputs {
            return Err(BtcError::InvalidTransaction);
            }

        self.mempool.push((Utc::now(), transaction));

        // sort by miner fee

        self.mempool.sort_by_key( |( _, transaction) | {


            let all_inputs  = transaction.inputs
                .iter()
                .map(|input| {
                    self.utxos
                        .get(&input.prev_transaction_output_hash)
                        .expect("Bug: impossible")
                        .1  
                        .value


                })
                .sum::<u64>();

            let all_outputs: u64 = transaction.outputs
                .iter()
                .map(|output  | output.value)
                .sum();

            let miner_fee = all_inputs - all_outputs;

            miner_fee
            
        });

        Ok(())
    }




    // cleanup mempool - remove transaction older than Max mempool age defined in lib.rs


    pub fn cleanup_mempool (&mut self) {

        let now = Utc::now();

        let mut utxo_hashes_to_unmark: Vec<Hash> = vec![];

        self.mempool.retain(|(timestamp, transaction )| {

            if now - *timestamp > chrono::Duration::seconds(crate::MAX_MEMPOOL_TRANSACTION_AGE as i64) {

                // push all  the utxo to unmarke to the vector
                // so we can unmark them later


                utxo_hashes_to_unmark.extend(transaction.inputs.iter().map(|input| {    

                        input.prev_transaction_output_hash

                }));
                false
            } else  {

                true
            }


        });

        // unmark all of the UTXOS


        for hash in utxo_hashes_to_unmark {

            self.utxos.entry(hash).and_modify(

                |(marked, _)| {

                    *marked = false;
                },
            );
        }




    }






    pub fn rebuild_utxos(&mut self)  {

        for block in &self.blocks {

            for transaction in &block.transactions {

                for input in &transaction.inputs {

                    self.utxos.remove(&input.prev_transaction_output_hash);
                }

                for output in transaction.outputs.iter() {

                    self.utxos.insert(output.hash(), (false, output.clone()));   
                }
            }
        }
    }




    pub fn add_block(&mut self, block: Block) -> Result<()> {

         
         if self.blocks.is_empty() {

            if block.header.prev_block_hash != Hash::zero() {

                println!("zero hash");

                return Err(BtcError::InvalidBlock);
            }
         } else {

                let last_block = self.blocks.last().unwrap();

                if block.header.prev_block_hash != last_block.hash() {

                    println!("prev hash is wrong");
                    return Err(BtcError::InvalidBlock);
                }

                if !block.header.hash().matches_target(block.header.target) {

                    println!("does not match target");
                    return Err(BtcError::InvalidBlock);

                }

                let calculated_merkle_root = MerkleRoot::calculate(&block.transactions);

                if calculated_merkle_root != block.header.merkle_root {

                    println!("invalid merkle root");
                    return Err(BtcError::InvalidBlock);
                }

                if block.header.timestamp <= last_block.header.timestamp {

                   block.verify_transaction(self.blocks_height(), &self.utxos)?;
                }


         }

         // Remove transaction from mempool that are now in the block
        
        let block_transactions: HashSet<_> = block.transactions.iter().map(|tx| tx.hash()).collect();

        self.mempool.retain(| (_,  tx)|  {

            !block_transactions.contains(&tx.hash())

        });
        
        self.blocks.push(block);
        self.try_adjust_target();
        Ok(())
    }


    pub fn blocks_height(&self) -> u64 {
        
        self.blocks.len() as u64
    }

    pub fn try_adjust_target(&mut self) {

        if self.blocks.is_empty() {
            
            return ;
        }

        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INTERVAL as usize != 0 {

            return;

        }

        let start_time = self.blocks[self.blocks.len() - crate::DIFFICULTY_UPDATE_INTERVAL as usize].header.timestamp;

        let end_time = self.blocks.last().unwrap().header.timestamp;

        let time_diff = end_time - start_time;

        let time_diff_seconds  = time_diff.num_seconds();

        let target_seconds = crate::IDEAL_BLOCK_TIME * crate::DIFFICULTY_UPDATE_INTERVAL;


        // multiply the current target by the actual time divided by the ideal time
        // Target is difficulty 

        let new_target = BigDecimal::parse_bytes(&self.target.to_string().as_bytes(), 10).expect("Bug: impossible")
                                                    * (BigDecimal::from(time_diff_seconds) / BigDecimal::from(target_seconds));

        // cut off decimal point and everything after it from string representation of new target


        let new_target_str = new_target.to_string().split('.').next().expect("expected a decimal point").to_owned();

        let new_target: U256 = U256::from_str_radix(&new_target_str, 10).expect("Bug: Impossible");

        // clamp new_target to be within the range of 4 * self.target adn self.target / 4 

        let new_target = if new_target < self.target / 4 {

            self.target / 4

        }  else if new_target > self.target * 4 {
             
            self.target * 4

        } else {

            new_target
        };

        // finally, we need to ensure that we do not decrease the target below minimum target

        // if the new target is more than the min-target , set it to the minimum target

        self.target = new_target.min(crate::MIN_TARGET);






    }

}


impl Saveable for Blockchain {


    fn load<I: Read>(reader: I) -> IoResult<Self> {


        ciborium::de::from_reader(reader).map_err(|_| {

            IoError::new(IoErrorKind::InvalidData,
             "failed to deserialize the data"
            )
        }) 
    }


    fn save<O: Write>(&self, writer: O) -> IoResult<()> {

        ciborium::ser::into_writer(self, writer).map_err(|_| {

            IoError::new(IoErrorKind::InvalidData,
             "failed to serialize the data"
            )
        }) 
    }



}