use core::fmt;

use crate::U256;

use serde::{Deserialize, Serialize};
use sha256::digest;


#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq,Eq, Hash)]
pub struct Hash(U256);

impl Hash {

    pub fn hash<T: serde::Serialize>(data: &T) -> Self {


        let mut serialized: Vec<u8> = vec![];

        if let Err(e) = ciborium::into_writer(data, &mut serialized) {

            panic!("failed to serialized the data {}", e);
        }

        let hash = digest(&serialized);

        let hash_bytes: Vec<u8>= hex::decode(hash).unwrap();

        let hash_array:[u8; 32]  =  hash_bytes.as_slice().try_into().unwrap();


        // @note check this , it can cause the problem. big_endian method, which interprets the array directly
        let x = U256::from_big_endian(&hash_array);



        Hash(x)




    }

    // the network difficulty sets the target and for a hash to be valid for a mined block
    // the hash has to be smaller number than the target
    pub fn matches_target(&self, target: U256) -> bool {

        self.0 <= target
    }


    pub fn zero() -> Self {

        Hash(U256::zero())
    }


    pub fn as_bytes(&self) -> [u8;32] {

        let mut bytes = vec![0; 32];

        //self.0.to_little_endian(&mut bytes );   changing this code because the to_little_endian() expect only one argument

        for (i, word) in self.0.0.iter().enumerate() {
            bytes[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
        }
        
        bytes.as_slice().try_into().unwrap()
    }
 

}



impl fmt::Display for Hash {

   

//     f is an instance of std::fmt::Formatter, which manages where the output is written. It is passed into the fmt function when implementing Display or Debug.
//     write! writes the formatted output to this formatter.

// "{:x}":

//     This is a formatting string. The {:x} means "format the value as a hexadecimal number in lowercase".
//     In Rust, format specifiers like x are used for hexadecimal formatting.
//         x: Lowercase hexadecimal.
//         X: Uppercase hexadecimal.

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        

        write!(f, "{:x}", self.0)
        
    }
}