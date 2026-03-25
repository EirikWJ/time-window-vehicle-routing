#![allow(dead_code)]

use rand::seq::SliceRandom;

use crate::{ga_params::GaParams, utils::get_2idx};

#[derive(Debug, Clone, Copy)]
pub enum MutationType {
    Insert,
    Scramble,
    Inversion,
    Swap,
}

// mutate(&mut arr, 1.0, MutationType::Swap);
pub fn mutate(genotype: &mut Vec<u32>, rate: f32, method: MutationType) -> Vec<u32> {
    if rand::random::<f32>() > rate {
        return genotype.clone();
    }
    let (ia, ib) = get_2idx(genotype.len() as usize);
    match method {
        // Insert mutation (123456 -> 125346)
        MutationType::Insert => {
            let value = genotype.remove(ib);
            genotype.insert(ia, value);
        }
        // Scramble mutation (123456 -> 135426)
        MutationType::Scramble => {
            let mut rng = rand::thread_rng();
            genotype[ia..=ib].shuffle(&mut rng);
        }
        // Inversion mutation (123456 -> 154326)
        MutationType::Inversion => {
            genotype[ia..=ib].reverse();
        }
        // Swap mutation (123456 -> 163452)
        MutationType::Swap => {
            genotype.swap(ia, ib);
        }
    }
    genotype.clone()
}

pub fn apply_mutators(genotype: &mut Vec<u32>, params: &GaParams) {
    *genotype = mutate(genotype, params.mutation_rate, params.primary_mutator);

    if let Some(method) = params.secondary_mutator {
        *genotype = mutate(
            genotype,
            params.mutation_rate * params.secondary_mutation_scale,
            method,
        );
    }
}