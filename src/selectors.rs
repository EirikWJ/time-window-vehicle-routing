use rand::{Rng, thread_rng};

use crate::{
    assignment_cache::{assignment_distance, build_assignment_cache},
    individual::Individual,
};

/// Parent selection using tournament with replacement.
/// Returns indices for the chosen parents.
pub fn tournament_selection(population: &[Individual], sample_size: usize) -> Vec<usize> {
    let pop_size = population.len();
    let mut parents = Vec::with_capacity(pop_size);
    let mut rng = thread_rng();

    for _ in 0..pop_size {
        // Pick first competitor
        let mut best_idx = rng.gen_range(0..pop_size);
        let mut best_fitness = population[best_idx].fitness;

        // Compete with 'sample_size' other competitors
        for _ in 1..sample_size {
            let comp_idx = rng.gen_range(0..pop_size);
            if population[comp_idx].fitness < best_fitness {
                best_idx = comp_idx;
                best_fitness = population[comp_idx].fitness;
            }
        }
        parents.push(best_idx);
    }
    parents
}

/// Crowding with direct parent child competition
pub fn crowding(
    parents: &Vec<Individual>,
    children: &Vec<Individual>,
    n_patients: usize,
    phi: f64,
) -> Vec<Individual> {
    let mut next_gen: Vec<Individual> = Vec::with_capacity(parents.len());
    let mut rng = rand::thread_rng();

    let mut i = 0;
    while i < parents.len() {
        let p1 = &parents[i];
        let p2 = &parents[i + 1];
        let c1 = &children[i];
        let c2 = &children[i + 1];

        // caches to measure similarity between individuals
        let p1c = build_assignment_cache(p1, n_patients);
        let p2c = build_assignment_cache(p2, n_patients);
        let c1c = build_assignment_cache(c1, n_patients);
        let c2c = build_assignment_cache(c2, n_patients);

        // choose pairing that minimizes total distance
        let same = assignment_distance(&c1c, &p1c, n_patients)
            + assignment_distance(&c2c, &p2c, n_patients);
        let cross = assignment_distance(&c1c, &p2c, n_patients)
            + assignment_distance(&c2c, &p1c, n_patients);

        // generalised crowding competition between pairs
        if same <= cross {
            next_gen.push(compete_pair(p1, c1, phi, &mut rng));
            next_gen.push(compete_pair(p2, c2, phi, &mut rng));
        } else {
            next_gen.push(compete_pair(p2, c1, phi, &mut rng));
            next_gen.push(compete_pair(p1, c2, phi, &mut rng));
        }

        i += 2;
    }

    next_gen
}

fn compete_pair(
    parent: &Individual,
    child: &Individual,
    phi: f64,
    rng: &mut impl Rng,
) -> Individual {
    let fp = parent.fitness as f64;
    let fc = child.fitness as f64;

    let p_take = generalised_crowding(fp, fc, phi);
    let u: f64 = rng.gen_range(0.0..1.0);

    if u < p_take {
        child.clone()
    } else {
        parent.clone()
    }
}

pub fn generalised_crowding(fp: f64, fc: f64, phi: f64) -> f64 {
    if fc < fp {
        fp / (fp + (phi * fc))
    } else if fc == fp {
        0.5
    } else {
        (phi * fp) / ((phi * fp) + fc)
    }
}
