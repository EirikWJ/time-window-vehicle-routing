use crate::crossovers::CrossoverType;
use crate::mutators::MutationType;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum SurvivorSelector {
    MuPlusLambda,
    Crowding { phi: f64 },
}

#[derive(Debug, Clone, Copy)]
pub struct GaParams {
    pub num_runs: usize,
    pub pop_size: u32,
    pub generations: u32,

    pub crossover_rate: f32,
    pub crossover_selector: CrossoverType,

    pub mutation_rate: f32,
    pub primary_mutator: MutationType,
    pub secondary_mutator: Option<MutationType>,// for lowering  selection pressure/ more exploration
    pub secondary_mutation_scale: f32, // typically 0.5 if there are two

    pub tournament_size: usize,
    pub survivor_selector: SurvivorSelector,

    // inserts random feasable individuals into population
    pub stagnation_for_injection: usize,
    // pop.size() / "injection_divisor" -> the number of random feasable individuals injected
    // when fitness improvement stagnates
    pub injection_divisor: u32,
    // what is recognised as a stupidly high fitness, and therefore not worthy of the computers CPU time. 
    // tldr; high fitness -> no need for local search
    pub feasible_fitness_threshold: f32, 

    pub init_restarts: usize,
    pub init_seed_top_k: usize,
    pub init_candidate_sample: usize,
    pub init_extra_positions: usize,
}
