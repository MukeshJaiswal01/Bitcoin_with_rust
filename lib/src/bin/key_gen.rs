use std::env;
use lib::crypto::PrivateKey;
use lib::util::Saveable;



fn main() {
    let name = env::args().nth(1).expect("Please provide a name");

    let private_key = PrivateKey::new_key();

    let public_key = private_key.public_key();


    // make a deep copy of the value, meaning it creates a new instance of the data rather 
    // than just copying a reference or pointer.
    // This prevents name from being consumed and lets you continue using it afterward. Without .clone(), the ownership of name would be transferred, and 
    // you wouldn’t be able to use name again in subsequent code.



    let public_key_file = name.clone() + ".pub.pem";

    let private_key_file = name + ".priv.cbor";

    private_key.save_to_file(&private_key_file).unwrap();

    public_key.save_to_file(&public_key_file).unwrap();

}