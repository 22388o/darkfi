use darkfi::{
    crypto::{
        keypair::PublicKey,
        proof::{ProvingKey, VerifyingKey},
        util::{mod_r_p, pedersen_commitment_scalar, pedersen_commitment_u64},
        Proof,
    },
    zk::vm::{Witness, ZkCircuit},
    zkas::decoder::ZkBinary,
    Result,
};
use halo2_gadgets::primitives::{
    poseidon,
    poseidon::{ConstantLength, P128Pow5T3},
};
use log::info;
use pasta_curves::{
    arithmetic::CurveAffine,
    group::{ff::Field, Curve},
    pallas,
};
use rand::rngs::OsRng;
use simplelog::{ColorChoice::Auto, Config, LevelFilter, TermLogger, TerminalMode::Mixed};

fn main() -> Result<()> {
    let loglevel = match option_env!("RUST_LOG") {
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        Some(_) | None => LevelFilter::Info,
    };
    TermLogger::init(loglevel, Config::default(), Mixed, Auto)?;

    /* ANCHOR: main */
    let bincode = include_bytes!("mint.zk.bin");
    let zkbin = ZkBinary::decode(bincode)?;

    // ======
    // Prover
    // ======

    // Witness values
    let value = 42;
    let token_id = pallas::Base::from(22);
    let value_blind = pallas::Scalar::random(&mut OsRng);
    let token_blind = pallas::Scalar::random(&mut OsRng);
    let serial = pallas::Base::random(&mut OsRng);
    let coin_blind = pallas::Base::random(&mut OsRng);
    let public_key = PublicKey::random(&mut OsRng);
    let coords = public_key.0.to_affine().coordinates().unwrap();

    let prover_witnesses = vec![
        Witness::Base(Some(*coords.x())),
        Witness::Base(Some(*coords.y())),
        Witness::Base(Some(pallas::Base::from(value))),
        Witness::Base(Some(token_id)),
        Witness::Base(Some(serial)),
        Witness::Base(Some(coin_blind)),
        Witness::Scalar(Some(value_blind)),
        Witness::Scalar(Some(token_blind)),
    ];

    // Create the public inputs
    let msgs = [*coords.x(), *coords.y(), pallas::Base::from(value), token_id, serial, coin_blind];
    let coin = poseidon::Hash::<_, P128Pow5T3, ConstantLength<6>, 3, 2>::init().hash(msgs);

    let value_commit = pedersen_commitment_u64(value, value_blind);
    let value_coords = value_commit.to_affine().coordinates().unwrap();

    let token_commit = pedersen_commitment_scalar(mod_r_p(token_id), token_blind);
    let token_coords = token_commit.to_affine().coordinates().unwrap();

    let public_inputs =
        vec![coin, *value_coords.x(), *value_coords.y(), *token_coords.x(), *token_coords.y()];

    // Create the circuit
    let circuit = ZkCircuit::new(prover_witnesses, zkbin.clone());

    info!(target: "PROVER", "Building proving key and creating the zero-knowledge proof");
    let proving_key = ProvingKey::build(11, &circuit);
    let proof = Proof::create(&proving_key, &[circuit], &public_inputs, &mut OsRng)?;

    // ========
    // Verifier
    // ========

    // Construct empty witnesses
    let verifier_witnesses = vec![
        Witness::Base(None),
        Witness::Base(None),
        Witness::Base(None),
        Witness::Base(None),
        Witness::Base(None),
        Witness::Base(None),
        Witness::Scalar(None),
        Witness::Scalar(None),
    ];

    // Create the circuit
    let circuit = ZkCircuit::new(verifier_witnesses, zkbin);

    info!(target: "VERIFIER", "Building verifying key and verifying the zero-knowledge proof");
    let verifying_key = VerifyingKey::build(11, &circuit);
    proof.verify(&verifying_key, &public_inputs)?;
    /* ANCHOR_END: main */

    Ok(())
}
