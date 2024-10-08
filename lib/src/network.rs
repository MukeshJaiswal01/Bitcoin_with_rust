
use serde::{Deserialize, Serialize};
use crate::crypto::PublicKey;
use crate::types::{Block, Transaction, TransactionOutput};
use std::io::{Error as IoError, Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};


#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {

    // fetch all UTXOs, belonging to a public key 
    FetchUTXOS(PublicKey),

    // UTXOS belonging to public key , bool determines if marked
    UTXOS(Vec<(TransactionOutput, bool)>),

    // Send a transaction to the network
    SubmitTransaction(Transaction),

    // Broadcast a new transaction to other nodes
    NewTransaction(Transaction),

    // Ask the node to prepare the optimal block template 
    // with the coinbase transaction paying the specified public key
    FetchTemplate(PublicKey),

    // The Template
    Template(Block),

    // Ask the node to validate a block template.
    // this is to prevent the node from mining an invalid block (e.g. if one has been found in the meantime,
    /// or if transactions have been removed from the mempool)
    /// 
    ValidateTemplate(Block),

    // If template is Valid
    TemplateValidity(bool),

    // Submit a mined block to a node
    SubmitTemplate(Block),

    // Ask node to report all other nodes it knows about
    DiscoverNodes,

    // This is the response to DiscoverNodes
    NodeList(Vec<String>),

    // Ask a node whats the highest block it knows about in comparison to the local blockchain
    AskDifference(u32),

    // This is the response to AskDifference
    Difference(i32),

    // Ask a node to send a block with the specified height
    FetchBlock(usize),

    // Broadcast a new block to other nodes
    NewBlock(Block), 

}



// We are going to use length-prefixed encoding for message and we are going to use ciborium for serialization

impl Message {

    pub fn encode(&self) -> Result<Vec<u8>, ciborium::ser::Error<IoError>> {

        let mut bytes = Vec::new();

        ciborium::into_writer(self, &mut bytes)?;

        Ok(bytes)
    }

    pub fn decode(data: &[u8]) -> Result<Self, ciborium::de::Error<IoError>> {

        ciborium::from_reader(data)


    }

    pub async fn send_async(&self, stream: &mut( impl AsyncWrite + Unpin)) -> Result<(), ciborium::ser::Error<IoError>> {

        let bytes = self.encode()?;

        let len = bytes.len() as u64;

        stream.write_all(&len.to_be_bytes()).await?;

        stream.write_all(&bytes).await?;

        Ok(())
    }


    pub async  fn recieve_asynce(stream: &mut (impl AsyncRead + Unpin)) -> Result<Self, ciborium::de::Error<IoError>> {


        let mut len_bytes= [0u8; 8];

        stream.read_exact(&mut len_bytes).await?;

        let len = u64::from_be_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];

        stream.read_exact(&mut data).await?;

        Self::decode(&data)

    }
    
}

