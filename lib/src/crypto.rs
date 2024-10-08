// ECDSA (Elliptic Curve Digital Signature Algorithm), and we can do the same.
// ECDSA can use different elliptical curve parameters, and we can use the so-called
// secp256k1, just like bitcoin.
// We need to import the ecdsa and k256 libraries. There is a mechanism similar to
// the one we have seen with serde and ciborium. The ecdsa crate is generic over
// the elliptical curve used, and the curve (just like the format in serde), needs to be
// supplied via another crate.



use spki::EncodePublicKey;
use crate::util::Saveable;
use ecdsa:: {
    signature::Signer,
    Signature as ECDSASignature, 
    SigningKey,
    VerifyingKey,
};

use std::io::{
    Error as IoError, ErrorKind as IoErrorKind, Read,
    Result as IoResult, Write,
    };

use crate::sha256::Hash;
use ecdsa::signature::Verifier;

use k256::Secp256k1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Signature(pub ECDSASignature<Secp256k1>);

impl Signature {

    pub fn sign_output(output_hash: &Hash, private_key: &PrivateKey) -> Self {


        let signing_key = &private_key.0;

        let signature = signing_key.sign(&output_hash.as_bytes());
        
        Signature(signature)
    }



    pub fn verify( &self, output_hash: &Hash, public_key: &PublicKey, ) -> bool {

        public_key.0.verify(&output_hash.as_bytes(), &self.0).is_ok()

        }
        



}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq,)]
pub struct PublicKey(VerifyingKey<Secp256k1>);


// save and load as PEM 
// PEM (Privacy-Enhanced Mail) is a widely used format for encoding cryptographic keys, 
//certificates, and other data in a Base64-encoded format, with human-readable headers and footers. While it originated in the context of secure email, it's now commonly used in many cryptographic systems to represent and store keys and certificates.

impl Saveable for PublicKey {

    fn load<I: Read>(mut reader: I) -> IoResult<Self> {
        
        // read PEM encoded public key into string

        let mut buff = String::new();
        reader.read_to_string(&mut buff)?;

        // decode the the public key from PEM

        let public_key = buff.parse().map_err(|_| {

            IoError::new(
                IoErrorKind::InvalidData,
               "failed to parse PublicKey",
           )
        })?;

        Ok(PublicKey(public_key))


    }


    fn save<O: Write>(&self, mut writer: O) -> IoResult<()> {

        let s = self.0.to_public_key_pem(Default::default()).map_err(|_| {

            IoError::new(
                IoErrorKind::InvalidData,
                "failed to serialize public key"

            )
        })?;
        

        writer.write_all(s.as_bytes())?;
        Ok(())


        
        
    }
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateKey(

    #[serde(with = "signkey_serde")]
    pub SigningKey<Secp256k1>,
);


impl PrivateKey {

    pub fn new_key() -> Self{

        PrivateKey(SigningKey::random(&mut rand::thread_rng()))
    }

    pub fn public_key(&self) -> PublicKey{

        PublicKey(self.0.verifying_key().clone())
    }
}

impl Saveable for PrivateKey{

    fn load<I: Read>(reader: I) -> IoResult<Self> {

        ciborium::de::from_reader(reader).map_err(|_| {

                IoError::new(
                IoErrorKind::InvalidData,
                "failed to deserialize PrivateKey"
                )

                
            
        })
    }


    fn save<O: Write>(&self, writer: O) -> IoResult<()> {

        ciborium::ser::into_writer(self, writer).map_err( |_| {

                IoError::new(
                IoErrorKind::InvalidData,
                "Failed to serialize PrivateKey",
                )
            },)?;

        Ok(())
     }
}

mod signkey_serde {

    use serde::Deserialize;

    pub fn serialize<S>(
        key: &super::SigningKey<super::Secp256k1>, 
        serializer: S
    ) -> Result<S::Ok, S::Error>
    where 
        S: serde::Serializer{

            serializer.serialize_bytes(&key.to_bytes())
        } 


     pub fn deserialize<'de, D>(
        deserializer: D,
        ) -> Result<super::SigningKey<super::Secp256k1>, D::Error>
         where
            D: serde::Deserializer<'de>,
        {
           let bytes: Vec<u8> =
            Vec::<u8>::deserialize(deserializer)?;
            Ok(super::SigningKey::from_slice(&bytes).unwrap())
       }
}




