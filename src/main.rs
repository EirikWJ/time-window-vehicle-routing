/*
Contributing Students:
Eirik Wang Johansen - 151232
Jens Kulås - 561835
*/

mod assignment_cache;
mod create_feasible_routes;
mod crossovers;
mod depot;
mod export;
mod ga_params;
mod individual;
mod instance;
mod kmeans;
mod load;
mod local_search;
mod mutators;
mod patient;
mod selectors;
mod utils;
mod fitness;

use crossovers::CrossoverType;
use ga_params::{GaParams, SurvivorSelector};
use load::load_instance;
use mutators::MutationType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./datasets/test_instances/test_instance_1.json".to_string());

    let mut instance = load_instance(&instance_path)?;

    let ga_params = GaParams {
        num_runs: 1,
        pop_size: 100,
        generations: 100,

        crossover_rate: 0.9,
        crossover_selector: CrossoverType::EdgeRecombination,
        mutation_rate: 0.35,
        secondary_mutation_scale: 0.5,
        primary_mutator: MutationType::Inversion,
        secondary_mutator: Some(MutationType::Swap),

        tournament_size: 4,

        survivor_selector: SurvivorSelector::Crowding { phi: 0.0 },
        stagnation_for_injection: 15,
        injection_divisor: 10,
        feasible_fitness_threshold: 1.0e11,

        init_restarts: 80,
        init_seed_top_k: 20,
        init_candidate_sample: 80,
        init_extra_positions: 6,
    };

    instance.run(ga_params);

    Ok(())
}
