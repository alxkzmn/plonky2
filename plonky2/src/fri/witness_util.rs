use anyhow::{anyhow, Result};
use itertools::Itertools;
use plonky2_field::types::Field;

use crate::field::extension::Extendable;
use crate::fri::proof::{FriProof, FriProofTarget};
use crate::hash::hash_types::{HashOut, RichField};
use crate::iop::witness::WitnessWrite;
use crate::plonk::config::AlgebraicHasher;

/// Set the targets in a `FriProofTarget` to their corresponding values in a `FriProof`.
pub fn set_fri_proof_target<F, W, H, const D: usize>(
    witness: &mut W,
    fri_proof_target: &FriProofTarget<D>,
    fri_proof: &FriProof<F, H, D>,
) -> Result<()>
where
    F: RichField + Extendable<D>,
    W: WitnessWrite<F> + ?Sized,
    H: AlgebraicHasher<F>,
{
    witness.set_target(fri_proof_target.pow_witness, fri_proof.pow_witness)?;

    let target_len = fri_proof_target.final_poly.0.len();
    let coeffs_len = fri_proof.final_poly.coeffs.len();

    if target_len < coeffs_len {
        return Err(anyhow!(
            "fri_proof->final_poly's target length is less than the proof length"
        ));
    }

    // Set overlapping elements
    for i in 0..coeffs_len {
        witness.set_extension_target(
            fri_proof_target.final_poly.0[i],
            fri_proof.final_poly.coeffs[i],
        )?;
    }

    // Set remaining elements in target to ZERO if target is longer
    for i in coeffs_len..target_len {
        witness.set_extension_target(fri_proof_target.final_poly.0[i], F::Extension::ZERO)?;
    }

    for (t, x) in fri_proof_target
        .commit_phase_merkle_caps
        .iter()
        .zip_eq(&fri_proof.commit_phase_merkle_caps)
    {
        witness.set_cap_target(t, x)?;
    }

    for (qt, q) in fri_proof_target
        .query_round_proofs
        .iter()
        .zip_eq(&fri_proof.query_round_proofs)
    {
        for (at, a) in qt
            .initial_trees_proof
            .evals_proofs
            .iter()
            .zip_eq(&q.initial_trees_proof.evals_proofs)
        {
            for (&t, &x) in at.0.iter().zip_eq(&a.0) {
                witness.set_target(t, x)?;
            }
            let target_len = at.1.siblings.len();
            let siblings_len = a.1.siblings.len();

            if target_len < siblings_len {
                return Err(anyhow!("fri_proof->query_round_proofs->initial_trees_proof->evals_proofs->siblings' target length is less than the proof length"));
            }

            // Set overlapping elements
            for i in 0..siblings_len {
                witness.set_hash_target(at.1.siblings[i], a.1.siblings[i])?;
            }

            // Set remaining elements in target to ZERO if target is longer
            for i in siblings_len..target_len {
                witness.set_hash_target(at.1.siblings[i], HashOut::ZERO)?;
            }
        }

        for (st, s) in qt.steps.iter().zip_eq(&q.steps) {
            for (&t, &x) in st.evals.iter().zip_eq(&s.evals) {
                witness.set_extension_target(t, x)?;
            }

            let target_len = st.merkle_proof.siblings.len();
            let siblings_len = s.merkle_proof.siblings.len();

            if target_len < siblings_len {
                return Err(anyhow!("fri_proof->query_round_proofs->steps->merkle_proof->siblings' target length is less than the proof length"));
            }

            // Set overlapping elements
            for i in 0..siblings_len {
                witness.set_hash_target(st.merkle_proof.siblings[i], s.merkle_proof.siblings[i])?;
            }

            // Set remaining elements in target to ZERO if target is longer
            for i in siblings_len..target_len {
                witness.set_hash_target(st.merkle_proof.siblings[i], HashOut::ZERO)?;
            }
        }
    }

    Ok(())
}
