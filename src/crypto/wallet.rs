use alloy_primitives::{hex, Address, Keccak256};
use k256::ecdsa::{SigningKey, VerifyingKey};
use rand_core::OsRng;

pub struct Wallet {
    pub private_key: SigningKey,
    pub public_key: VerifyingKey,
    pub address: Address,
}

fn to_address(verifying_key: VerifyingKey) -> Address {
    let public_key_bytes = verifying_key.to_encoded_point(false).as_bytes().to_vec();

    // Compute Ethereum address
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]); // Skip first byte of public key (0x04 prefix)
    let hash = hasher.finalize();
    // Ethereum address is last 20 bytes of hash
    Address::from_slice(&hash[12..])
}

impl Wallet {
    /// Constructor that initializes an Ethereum wallet with provided signing and verifying keys
    pub fn new(signing_key: SigningKey, verifying_key: VerifyingKey) -> Self {
        Wallet {
            private_key: signing_key,
            public_key: verifying_key,
            address: to_address(verifying_key),
        }
    }

    /// Generate a new Ethereum wallet
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng); // Generate random private key
        let verifying_key = VerifyingKey::from(&signing_key); // Derive public key
        let public_key_bytes = verifying_key.to_encoded_point(false).as_bytes().to_vec();

        // Compute Ethereum address
        let mut hasher = Keccak256::new();
        hasher.update(&public_key_bytes[1..]); // Skip first byte of public key (0x04 prefix)
        let hash = hasher.finalize();
        let eth_address = Address::from_slice(&hash[12..]); // Ethereum address is last 20 bytes of hash

        Wallet {
            private_key: signing_key,
            public_key: verifying_key,
            address: eth_address,
        }
    }

    /// Retrieve the private key as a hexadecimal string
    pub fn get_private_key(&self) -> String {
        let private_key_bytes = self.private_key.to_bytes();
        format!("0x{}", hex::encode(private_key_bytes))
    }

    /// Retrieve the public key as a hexadecimal string
    pub fn get_public_key(&self) -> String {
        let public_key_bytes = self.public_key.to_encoded_point(false).as_bytes().to_vec();
        format!("0x{}", hex::encode(public_key_bytes))
    }

    /// Retrieve the Ethereum address as a hexadecimal string
    pub fn get_address(&self) -> String {
        self.address.to_string()
    }
}
