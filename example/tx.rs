use incrementalmerkletree::{bridgetree::BridgeTree, Frontier, Tree};
use rand::rngs::OsRng;

use darkfi::{
    crypto::{
        coin::Coin,
        keypair::{Keypair, PublicKey, SecretKey},
        merkle_node::MerkleNode,
        note::{EncryptedNote, Note},
        nullifier::Nullifier,
        proof::{ProvingKey, VerifyingKey},
        token_id::generate_id2,
    },
    node::state::{state_transition, ProgramState, StateUpdate},
    tx,
    util::NetworkName,
    zk::circuit::{mint_contract::MintContract, spend_contract::SpendContract},
    Result,
};

struct MemoryState {
    // The entire merkle tree state
    tree: BridgeTree<MerkleNode, 32>,
    // List of all previous and the current merkle roots
    // This is the hashed value of all the children.
    merkle_roots: Vec<MerkleNode>,
    // Nullifiers prevent double spending
    nullifiers: Vec<Nullifier>,
    // All received coins
    // NOTE: we need maybe a flag to keep track of which ones are spent
    // Maybe the spend field links to a tx hash:input index
    // We should also keep track of the tx hash:output index where this
    // coin was received
    own_coins: Vec<(Coin, Note)>,
    mint_vk: VerifyingKey,
    spend_vk: VerifyingKey,

    // Public key of the cashier
    cashier_signature_public: PublicKey,
    // List of all our secret keys
    secrets: Vec<SecretKey>,
}

impl ProgramState for MemoryState {
    fn is_valid_cashier_public_key(&self, public: &PublicKey) -> bool {
        public == &self.cashier_signature_public
    }

    fn is_valid_merkle(&self, merkle_root: &MerkleNode) -> bool {
        self.merkle_roots.iter().any(|m| m == merkle_root)
    }

    fn nullifier_exists(&self, nullifier: &Nullifier) -> bool {
        self.nullifiers.iter().any(|n| n == nullifier)
    }

    fn mint_vk(&self) -> &VerifyingKey {
        &self.mint_vk
    }

    fn spend_vk(&self) -> &VerifyingKey {
        &self.spend_vk
    }
}

impl MemoryState {
    fn apply(&mut self, mut update: StateUpdate) {
        // Extend our list of nullifiers with the ones from the update
        self.nullifiers.append(&mut update.nullifiers);

        // Update merkle tree and witnesses
        for (coin, enc_note) in update.coins.into_iter().zip(update.enc_notes.into_iter()) {
            // Add the new coins to the merkle tree
            let node = MerkleNode(coin.0);
            self.tree.append(&node);

            // Keep track of all merkle roots that have existed
            self.merkle_roots.push(self.tree.root());

            if let Some((note, _secret)) = self.try_decrypt_note(enc_note) {
                self.own_coins.push((coin, note));
                self.tree.witness();
            }
        }
    }

    fn try_decrypt_note(&self, ciphertext: EncryptedNote) -> Option<(Note, SecretKey)> {
        // Loop through all our secret keys...
        for secret in &self.secrets {
            // ... attempt to decrypt the note ...
            if let Ok(note) = ciphertext.decrypt(secret) {
                // ... and return the decrypted note for this coin.
                return Some((note, *secret))
            }
        }
        // We weren't able to decrypt the note with any of our keys.
        None
    }
}

fn main() -> Result<()> {
    let cashier_signature_secret = SecretKey::random(&mut OsRng);
    let cashier_signature_public = PublicKey::from_secret(cashier_signature_secret);

    let keypair = Keypair::random(&mut OsRng);

    const K: u32 = 11;
    let mint_vk = VerifyingKey::build(K, &MintContract::default());
    let spend_vk = VerifyingKey::build(K, &SpendContract::default());

    let mut state = MemoryState {
        tree: BridgeTree::<MerkleNode, 32>::new(100),
        merkle_roots: vec![],
        nullifiers: vec![],
        own_coins: vec![],
        mint_vk,
        spend_vk,
        cashier_signature_public,
        secrets: vec![keypair.secret],
    };

    let token_id =
        generate_id2("So11111111111111111111111111111111111111112", &NetworkName::Solana)?;

    let builder = tx::TransactionBuilder {
        clear_inputs: vec![tx::TransactionBuilderClearInputInfo {
            value: 110,
            token_id,
            signature_secret: cashier_signature_secret,
        }],
        inputs: vec![],
        outputs: vec![tx::TransactionBuilderOutputInfo {
            value: 110,
            token_id,
            public: keypair.public,
        }],
    };

    let mint_pk = ProvingKey::build(K, &MintContract::default());
    let spend_pk = ProvingKey::build(K, &SpendContract::default());
    let tx = builder.build(&mint_pk, &spend_pk)?;

    tx.verify(&state.mint_vk, &state.spend_vk).expect("tx verify");

    let _note = tx.outputs[0].enc_note.decrypt(&keypair.secret)?;

    let update = state_transition(&state, tx)?;
    state.apply(update);

    // Now spend
    let (coin, note) = &state.own_coins[0];
    let node = MerkleNode(coin.0);
    let (leaf_position, merkle_path) = state.tree.authentication_path(&node).unwrap();

    let builder = tx::TransactionBuilder {
        clear_inputs: vec![],
        inputs: vec![tx::TransactionBuilderInputInfo {
            leaf_position,
            merkle_path,
            secret: keypair.secret,
            note: *note,
        }],
        outputs: vec![tx::TransactionBuilderOutputInfo {
            value: 110,
            token_id,
            public: keypair.public,
        }],
    };

    let tx = builder.build(&mint_pk, &spend_pk)?;

    let update = state_transition(&state, tx)?;
    state.apply(update);

    Ok(())
}
