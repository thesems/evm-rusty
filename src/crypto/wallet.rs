use alloy_primitives::{hex, Address, Keccak256};
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand_core::OsRng;

/// Trait defining wallet functionality
pub trait Wallet {
    fn generate() -> Self;
    fn get_private_key(&self) -> String;
    fn get_public_key(&self) -> String;
    fn get_address_as_hex(&self) -> String;
    fn get_address(&self) -> &Address;
}

/// Struct representing an Ethereum Wallet
pub struct EthereumWallet {
    private_key: SigningKey,
    public_key: VerifyingKey,
    eth_address: Address,
}

impl Wallet for EthereumWallet {
    /// Generate a new Ethereum wallet
    fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng); // Generate random private key
        let verifying_key = VerifyingKey::from(&signing_key); // Derive public key
        let public_key_bytes = verifying_key.to_encoded_point(false).as_bytes().to_vec();

        // Compute Ethereum address
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]); // Skip first byte of public key (0x04 prefix)
        let hash = hasher.finalize();
        let eth_address = Address::from_slice(&hash[12..]); // Ethereum address is last 20 bytes of hash

        EthereumWallet {
            private_key: signing_key,
            public_key: verifying_key,
            eth_address,
        }
    }

    /// Retrieve the private key as a hexadecimal string
    fn get_private_key(&self) -> String {
        let private_key_bytes = self.private_key.to_bytes();
        format!("0x{}", hex::encode(private_key_bytes))
    }

    /// Retrieve the public key as a hexadecimal string
    fn get_public_key(&self) -> String {
        let public_key_bytes = self.public_key.to_encoded_point(false).as_bytes().to_vec();
        format!("0x{}", hex::encode(public_key_bytes))
    }

    /// Retrieve the Ethereum address as a hexadecimal string
    fn get_address_as_hex(&self) -> String {
        self.eth_address.to_string()
    }

    /// Retrieve the Ethereum address
    fn get_address(&self) -> &Address {
        &self.eth_address
    }
}
