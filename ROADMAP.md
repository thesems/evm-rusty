1. **Basic Transaction Processing**
    - [x] Simple value transfers between EOAs (no contract code)
    - [ ] Transaction validation
      - [ ] Check nonce
      - [ ] Check sender has enough balance for value + gas
      - [ ] Verify signature

2. **Basic Gas Handling**
    - Implement gas counting
    - Calculate transaction fees
    - Handle gas refunds
    - Pay fees to block fee recipient (former miner/coinbase)

3. **Contract Creation Transactions**
    - Deploy contract code
    - Initialize contract storage
    - Handle contract creation failures

4. **EVM Core (Smart Contract Execution)**
    - Implement basic stack machine
    - Start with simple opcodes like:
        - ADD, SUB, MUL, DIV
        - PUSH, POP, SWAP, DUP
        - MSTORE, MLOAD (memory operations)
        - SSTORE, SLOAD (storage operations)
    - Add more complex opcodes gradually

5. **Memory and Storage Management**
    - Implement linear memory model
    - Gas counting for memory expansion
    - Storage updates and gas refunds
    - Implement simple key-value storage first, optimize later

6. **Advanced Features**
    - Precompiled contracts
    - Self-destruct handling
    - Log generation (for event emission)
    - Implement EIP-1559 fee market
    - Access lists (EIP-2930)

7. **Optimizations and Advanced Data Structures**
    - Replace simple storage with Merkle Patricia Trie
    - Add caching layers
    - Optimize gas calculation
    - State management optimizations
