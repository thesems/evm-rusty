// EIP-2718 - multiple transaction formats via Recursive Length Prefix (RLP) encoding

use alloy_primitives::{Address, B256, Keccak256};
use k256::ecdsa::{
    hazmat::SignPrimitive,
    hazmat::VerifyPrimitive,
    signature::Verifier,
    RecoveryId, Signature, SigningKey, VerifyingKey,
};
use k256::ecdsa::signature::hazmat::PrehashVerifier;

pub const TRANSACTION_GAS_COST: u64 = 21000;
pub const GWEI_TO_WEI: u64 = 1_000_000_000;
pub const ETH_TO_WEI: u64 = GWEI_TO_WEI * 1_000_000_000;

pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub signature: [u8; 65],
    pub nonce: u64,
    pub value: u64,
    pub input_data: B256,
    // the maximum amount of gas units that can be consumed by the transaction.
    // The EVM specifies the units of gas required by each computational step
    pub gas_limit: u64,

    pub chain_id: u64,

    // EIP-2930
    // list of addresses and storage keys transaction intends to access
    // access_list: TBD

    // EIP-1559
    // the maximum price of the consumed gas to be included as a tip to the validator
    pub max_priority_fee_per_gas: u64,
    // the maximum fee per unit of gas willing to be paid for the transaction (inclusive of baseFeePerGas and maxPriorityFeePerGas)
    pub max_fee_per_gas: u64,
}

impl Transaction {
    pub fn new(
        from: Address,
        to: Address,
        value: u64,
        gas_limit: u64,
        max_priority_fee_per_gas: u64,
        max_fee_per_gas: u64,
    ) -> Self {
        Self {
            from,
            to,
            value,
            signature: [0u8; 65],
            nonce: 0,
            input_data: Default::default(),
            gas_limit,
            chain_id: 0,
            max_priority_fee_per_gas,
            max_fee_per_gas,
        }
    }

    // Calculate the hash that will be signed
    // This follows EIP-2718 and EIP-1559 transaction format
    pub fn hash_for_signing(&self) -> Vec<u8> {
        let mut hasher = Keccak256::new();

        // We use RLP encoding in practice, but for simplicity, we'll just concatenate fields
        // In a real implementation, you'd want to use proper RLP encoding here
        hasher.update(&[0x02]); // transaction type 2 (EIP-1559)
        hasher.update(&self.chain_id.to_be_bytes());
        hasher.update(&self.nonce.to_be_bytes());
        hasher.update(&self.max_priority_fee_per_gas.to_be_bytes());
        hasher.update(&self.max_fee_per_gas.to_be_bytes());
        hasher.update(&self.gas_limit.to_be_bytes());
        hasher.update(self.to.into_word().as_slice());
        hasher.update(&self.value.to_be_bytes());
        // In practice, we'd also include access_list and data field here

        hasher.finalize().to_vec()
    }

    pub fn sign(&mut self, private_key: &SigningKey) {
        // Sign and get recovery id
        let (signature, recovery_id) = private_key
            .sign_prehash_recoverable(self.hash_for_signing().as_slice())
            .expect("Signing failed");

        // Store signature with recovery id
        self.signature[..64].copy_from_slice(&signature.to_bytes());
        self.signature[64] = recovery_id.to_byte();
    }

    pub fn verify_signature(&self) -> bool {
        let recovery_id = RecoveryId::try_from(self.signature.as_slice()[64]).unwrap();
        let signature = Signature::from_slice(&self.signature.as_slice()[0..64]).unwrap();
        // let sig = signature.to_bytes().as_slice();
        if let Ok(verifying_key) = VerifyingKey::recover_from_prehash(
            self.hash_for_signing().as_slice(),
            &signature,
            recovery_id,
        ) {
            verifying_key
                .verify_prehash(self.hash_for_signing().as_slice(), &signature)
                .unwrap();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::wallet::{EthereumWallet, Wallet};

    #[test]
    fn test_sign_verify() {
        let eth_wallet = EthereumWallet::generate();
        let mut tx = Transaction::new(
            eth_wallet.address.clone(),
            eth_wallet.address.clone(),
            100,
            21000,
            100,
            100,
        );
        tx.sign(&eth_wallet.private_key);
        assert!(tx.verify_signature());
    }
}
