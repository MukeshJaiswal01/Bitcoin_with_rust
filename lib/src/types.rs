mod block;
mod blockchain;
mod transaction;


// To make the library compile again, we need to add a couple of pub use statements.
// This statement works as both an import and a re-export. 

pub use block::{Block, BlockHeader};
pub use blockchain::Blockchain;
pub use transaction::{
Transaction, TransactionInput, TransactionOutput,
};