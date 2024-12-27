use su_chain::crypto::wallet::Wallet;

fn main() {
    // Use Wallet trait to create a new Ethereum wallet
    let wallet = Wallet::generate();

    println!("Private Key: {}", wallet.get_private_key());
    println!("Public Key: {}", wallet.get_public_key());
    println!("Ethereum Address: {}", wallet.get_address());
}
