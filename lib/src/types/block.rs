use chrono::naive::serde::ts_microseconds::deserialize;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::{Transaction, TransactionOutput};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::U256;
use std::collections::HashMap;

use crate::util::Saveable;
use std::io::{
Error as IoError, ErrorKind as IoErrorKind, Read,
Result as IoResult, Write,
};






#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct  Block {

    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {

    pub fn new(header: BlockHeader, transaction: Vec<Transaction>)-> Self {

        Block{
            header: header,
            transactions: transaction,
        }
    }




    pub fn hash(&self) -> Hash{

        Hash::hash(self)
    }


    // verify the all transaction in the block 

    pub fn verify_transaction(&self, predicted_block_height: u64, utxos: &HashMap<Hash, (bool, TransactionOutput)>) -> Result<()> {

        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        // reject completely emtpy blocks

        if self.transactions.is_empty() {

            return Err(BtcError::InvalidTransaction);
        }

        self.verify_coinbase_transaction(predicted_block_height, utxos)?;

        for transaction in self.transactions.iter().skip(1)  {   // skipping the coinbase transaction 

            let mut input_value = 0;
            let mut output_value = 0;
             
            for input in &transaction.inputs {

                let prev_output = utxos.get(&input.prev_transaction_output_hash).map(|(_, output)| output);
                
                if prev_output.is_none() {
                    
                    return Err(BtcError::InvalidTransaction);
                }

                let prev_output = prev_output.unwrap();

                // prevent same-block double spending

                if inputs.contains_key(&input.prev_transaction_output_hash) {

                    return Err(BtcError::InvalidTransaction);
                }

                // check if the signature is valid

                if !input.signature.verify(&input.prev_transaction_output_hash, &prev_output.pubkey) {

                    return Err(BtcError::InvalidSignature);
                }

                input_value += prev_output.value;

                inputs.insert(input.prev_transaction_output_hash, prev_output.clone());


            }

            for output in &transaction.outputs {

                output_value += output.value;

            }

            // it is fine for output value to be less than input value as 
            // the difference between them will be equal to fee for the miner

            if input_value < output_value {

                return Err(BtcError::InvalidTransaction);

            }



        }

        Ok(())
    }

    
    // verify coinbase transaction 


    pub fn verify_coinbase_transaction(&self, predicted_block_height: u64, utxos: &HashMap<Hash, (bool, TransactionOutput)>) -> Result<()> {


        // coinbase tx is the first transaction in the block

        let coinbase_transaction = &self.transactions[0];

        if coinbase_transaction.inputs.len() == 0 {

            return Err(BtcError::InvalidTransaction);
        }

        if coinbase_transaction.outputs.len() == 0 {

            return Err(BtcError::InvalidTransaction);
        }


        let miner_fees = self.calculate_miner_fees(utxos)?;
        
        let block_reward = crate::INITIAL_REWARD * 10u64.pow(8) / 2u64.pow((predicted_block_height / crate::HALVING_INTERVAL) as u32);

        let total_coinbase_outputs: u64 = coinbase_transaction.outputs.iter().map(|output| output.value).sum();

        if total_coinbase_outputs != block_reward + miner_fees {

            return Err(BtcError::InvalidTransaction);
        }

        Ok(())
    }


    pub fn calculate_miner_fees(&self, utxos: &HashMap<Hash, (bool, TransactionOutput)>) -> Result<u64> {
         
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        // checks every transaction after coinbase

        for transaction in self.transactions.iter().skip(1) {

            for input in &transaction.inputs {

                // input dont contain the values of the outputs, so we need to match the inputs to output
                
                let prev_output = utxos.get(
                    &input.prev_transaction_output_hash,
                ).map(|(_, output)| output);

                if prev_output.is_none() {

                    return Err(BtcError::InvalidTransaction);
                }

                let prev_output = prev_output.unwrap();

                if inputs.contains_key(&input.prev_transaction_output_hash,) {

                    return Err(BtcError::InvalidTransaction);


                }

                inputs.insert(input.prev_transaction_output_hash, prev_output.clone());

            }

            for output in &transaction.outputs {

                if outputs.contains_key(&output.hash()) {

                    return Err(BtcError::InvalidTransaction);
                }

                outputs.insert( output.hash(),output.clone(), );
            }
        }


        let input_value:u64 = inputs.values().map(|output| output.value).sum();

        let output_value:u64 = outputs.values().map(|output| output.value).sum();
        
        Ok(input_value - output_value)

    }


            



}

impl Saveable for Block {

    fn load<I: Read>(reader: I) -> IoResult<Self> {

        // The map_err method is used to transform the Err variant of a Result. 
        // It's similar to map, which transforms the Ok variant, but map_err focuses on errors, 
        // allowing you to change the error type or value while leaving successful results (Ok) untouched.
        
        ciborium::de::from_reader(reader).map_err(|_| {
           

           // here map_err convert io::error to IoError 
            IoError::new(IoErrorKind::InvalidData, 
             "failed to deserialize data"
            )

        })
    }



    fn save<O: Write>(&self, writer: O) -> IoResult<()> {

        ciborium::ser::into_writer(self, writer).map_err( |_| {

                IoError::new(
                IoErrorKind::InvalidData,
                "Failed to serialize Block",
                )
            },
            )
        }



    }





#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {

    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: Hash,
    pub merkle_root: MerkleRoot,  // hash of the the merkle tree root derived from all of the transaction in this block
    pub target: U256,   // a number , which ha to higher than the hash of this block for it to considered valid

}

impl BlockHeader {


    pub fn new( timestamp: DateTime<Utc>, nonce:u64, prev_block_hash: Hash, merkle_root: MerkleRoot, target: U256) -> Self {
      

      BlockHeader {

        timestamp,
        nonce,
        prev_block_hash,
        merkle_root, 
        target,
      }

    }


    pub fn hash(&self) -> Hash {

        Hash::hash(self)
    }


    pub fn mine(&mut self, steps: usize) -> bool {

        // if the block already matches target, return early 

        if self.hash().matches_target(self.target) {

            return true;
        }

        for _ in  0..steps {

            if let Some(new_nonce) = self.nonce.checked_add(1) {  //  the checked_add() function is used to perform addition with overflow checking.

                self.nonce = new_nonce;

            } else {

                self.nonce = 0;
                self.timestamp = Utc::now()  // why using timestamp as nonce
            }
            if self.hash().matches_target(self.target) {

                return true;
            }


        }

        false
    }
}


