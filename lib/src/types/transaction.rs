use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;
use crate::util::Saveable;
use std::io::{
    Error as IoError, ErrorKind as IoErrorKind, Read,
    Result as IoResult, Write,
    };





#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {

    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,

}

impl Transaction {

    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {

        Transaction{
            inputs: inputs,
            outputs: outputs,
        }
    }

    pub fn hash(&self) -> Hash{

        Hash::hash(self)
    }
}


// save and load expecting CBOR form ciborium as format
impl Saveable for Transaction {

    fn load<I: Read>(reader: I) -> IoResult<Self> {

        ciborium::de::from_reader(reader).map_err(|_| {

            IoError::new(
                IoErrorKind::InvalidData,
                "failed to deserialize transaction"
            )
        })
    }


    fn save<O: Write>(&self, writer: O) -> IoResult<()> {


        ciborium::ser::into_writer(self, writer).map_err(|_| {

            IoError::new(IoErrorKind::InvalidData,
            "failed to serialize transaction",
            )
        })
    }


}




#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput{

    pub prev_transaction_output_hash: Hash,

    // @note replacing the script with simple signature field to make it simpler
    pub signature: crate::crypto::Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput{

    pub value: u64,
    pub unique_id: Uuid,
    pub pubkey: crate::crypto::PublicKey,


}

impl TransactionOutput {

    pub fn hash(&self) -> Hash {
    Hash::hash(self)

    }
}    

