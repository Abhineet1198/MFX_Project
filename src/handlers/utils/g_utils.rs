use ethers::signers::{LocalWallet, Signer};
use ethers::core::rand::thread_rng;
use ethers::utils::hex;

pub fn generate_wallet() -> (String, String) {
    let wallet = LocalWallet::new(&mut thread_rng());
    
    // Get address using the Signer trait
    let address = format!("{:?}", wallet.address());

    // Get private key
    let private_key = hex::encode(wallet.signer().to_bytes());

    (address, private_key)
}
