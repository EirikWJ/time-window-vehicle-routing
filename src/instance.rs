use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{SeedableRng, thread_rng};
use serde::Deserialize;
use std::collections::HashMap;

use crate::crossovers::crossover;
use crate::{depot::Depot, individual::Individual, patient::Patient};
use crate::export::export;
use crate::ga_params::{GaParams, SurvivorSelector};
use crate::kmeans::assign_clusters;
use crate::local_search::{
    INIT_LOCAL_SEARCH_PASSES, INJECTION_LOCAL_SEARCH_PASSES,
    improve_solution,
};
use crate::mutators::{apply_mutators};
use crate::selectors::{crowding, tournament_selection};
use crate::utils::{
    FitnessHistoryRow, deserialize_u32_from_number, flush_stdout, format_progress_bar,
    population_fitness_stats, population_route_edge_entropy, write_fitness_history_csv,
};

const PROGRESS_BAR_WIDTH: usize = 40;
const CLUSTER_SEED_EVERY: usize = 4;

#[derive(Debug, Deserialize)]
pub struct Instance {
    pub instance_name: String,
    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub nbr_nurses: u32,
    #[serde(deserialize_with = "deserialize_u32_from_number")]
    pub capacity_nurse: u32,
    pub benchmark: f32,
    pub depot: Depot,
    pub patients: HashMap<String, Patient>,
    pub travel_times: Vec<Vec<f64>>,

    #[serde(skip)]
    pub population: Vec<Individual>,
}

impl Instance {
    fn sort_population(&mut self) {
        self.population
            .sort_by(|a, b| a.fitness.total_cmp(&b.fitness));
    }


    /// Initialises the population 
    /// 
    /// Creates feasible individuals or random permutations as fallback
    /// Regularly creates individuals from k-means cluster to add diversity to the population
    fn init_pop(&mut self, params: &GaParams) {
        self.population.clear();

        let mut rng = thread_rng();
        let mut seeded_rng = StdRng::seed_from_u64(42);
        let clusters = assign_clusters(&self.patients, self.nbr_nurses as usize);

        let patient_ids: Vec<u32> = self
            .patients
            .keys()
            .filter_map(|id| id.parse::<u32>().ok())
            .collect();

        while self.population.len() < params.pop_size as usize {
            // Create feasible individual or random as fallback
            let mut individual = if let Some(ind) = self.init_feasible_individual(
                &mut seeded_rng,
                params.init_restarts,
                params.init_seed_top_k,
                params.init_candidate_sample,
                params.init_extra_positions,
            ) {
                ind
            } else {
                let mut genotype = patient_ids.clone();
                genotype.shuffle(&mut rng);

                let routes = self.decode_routes_from_permutation(&genotype);
                let fitness = self.calc_fitness(&genotype, &routes);

                Individual {
                    genotype,
                    fitness,
                    routes,
                }
            };

            // Create individual from k-means clusters
            if CLUSTER_SEED_EVERY > 0 && self.population.len() % CLUSTER_SEED_EVERY == 0 {
                let mut from_clusters = Vec::new();
                let mut packed = Vec::new();
                let mut start = 0usize;

                for cluster in &clusters {
                    let mut vals: Vec<u32> = cluster
                        .iter()
                        .filter_map(|id| id.parse::<u32>().ok())
                        .collect();

                    vals.shuffle(&mut rng);
                    if vals.is_empty() { continue; }

                    let end = start + vals.len() - 1;
                    from_clusters.extend(vals.iter().copied());
                    packed.push((start, end));
                    start = end + 1;
                }

                if !from_clusters.is_empty() {
                    let fit = self.calc_fitness(&from_clusters, &packed);
                    individual = Individual {
                        genotype: from_clusters,
                        fitness: fit,
                        routes: packed,
                    };
                }
            }

            // Use local search to improve individuals before adding them to population
            improve_solution(self, &mut individual, INIT_LOCAL_SEARCH_PASSES);
            self.population.push(individual);
        }

        self.sort_population();
    }

    /// Perform parent selection, crossover + mutation using the selected crossover and mutation options
    fn create_offspring(&self, params: &GaParams) -> (Vec<Individual>, Vec<Individual>) {
        let parent_idxs = tournament_selection(&self.population, params.tournament_size.max(1));
        let parent_pool: Vec<Individual> = parent_idxs
            .into_iter()
            .map(|idx| self.population[idx].clone())
            .collect();

        let mut selected_parents = Vec::with_capacity(params.pop_size as usize);
        let mut offspring = Vec::with_capacity(params.pop_size as usize);

        // Create children from every parent pair
        for pair in parent_pool.chunks(2) {
            if pair.len() < 2 {
                break;
            }

            selected_parents.push(pair[0].clone());
            selected_parents.push(pair[1].clone());

            let mut g1 = pair[0].genotype.clone();
            let mut g2 = pair[1].genotype.clone();

            // Crossover
            let (mut c1, mut c2) = crossover(
                &mut g1,
                &mut g2,
                params.crossover_rate,
                params.crossover_selector,
            );

            // Mutation
            apply_mutators(&mut c1, params);
            apply_mutators(&mut c2, params);

            // Calculate fitness of children and add to offspring
            offspring.push(self.evaluate_child(c1, params));
            offspring.push(self.evaluate_child(c2, params));
        }

        (selected_parents, offspring)
    }

    fn apply_mu_plus_lambda(&mut self, mut offspring: Vec<Individual>, pop_size: usize) {
        self.population.append(&mut offspring);
        self.sort_population();
        self.population.truncate(pop_size);
    }

    /// Perform survivor selection using the given survivorselector option
    fn select_survivors(
        &mut self,
        selected_parents: Vec<Individual>,
        offspring: Vec<Individual>,
        params: &GaParams,
    ) {
        let pop_size = params.pop_size as usize;

        match params.survivor_selector {
            // Mu + lambda selection
            SurvivorSelector::MuPlusLambda => {
                self.apply_mu_plus_lambda(offspring, pop_size);
            }
            // Generalised crowding
            SurvivorSelector::Crowding { phi } => {
                let mut parents = selected_parents;
                let mut children = offspring;

                let mut pair_len = parents.len().min(children.len());
                if pair_len % 2 == 1 {
                    pair_len -= 1;
                }

                if pair_len < 2 {
                    self.apply_mu_plus_lambda(children, pop_size);
                    return;
                }

                parents.truncate(pair_len);
                children.truncate(pair_len);

                let mut next_gen = crowding(&parents, &children, self.patients.len(), phi);

                // Copy the best from current population if next gen is too few
                if next_gen.len() < pop_size {
                    self.sort_population();
                    let missing = pop_size - next_gen.len();
                    next_gen.extend(self.population.iter().take(missing).cloned());
                }

                // Cut down if too big
                next_gen.sort_by(|a, b| a.fitness.total_cmp(&b.fitness));
                next_gen.truncate(pop_size);
                self.population = next_gen;
            }
        }
    }

    /// Injects a given amount of new feasible individuals into the population if they beat existing individuals
    fn inject_diversity(&mut self, attempt: usize, generation: u32, params: &GaParams) {
        let divisor = params.injection_divisor.max(1);
        let inject = (params.pop_size / divisor).max(1) as usize;
        let mut seeded_rng =
            StdRng::seed_from_u64(1000 + generation as u64 + (attempt as u64 * 10000));

        // Create feasible individuals
        for _ in 0..inject {
            if let Some(mut ind) = self.init_feasible_individual(
                &mut seeded_rng,
                params.init_restarts,
                params.init_seed_top_k,
                params.init_candidate_sample,
                params.init_extra_positions,
            ) {
                improve_solution(self, &mut ind, INJECTION_LOCAL_SEARCH_PASSES);
                self.population.push(ind);
            }
        }

        // Keep best from combination of population and potential injections
        self.sort_population();
        self.population.truncate(params.pop_size as usize);
    }

    /// Runs the genetic algorithm
    /// 
    /// Stores fitness progression, Initialises the population, performs parent selection, crossover, mutation, survivor selection.
    /// Injects new feasible individuals if progress stagnates.
    /// 
    /// Saves the fitness history and results to file.
    pub fn run(&mut self, params: GaParams) {
        let num_runs = params.num_runs.max(1);
        let mut best_global: Option<Individual> = None;
        let mut fitness_history = Vec::with_capacity(num_runs * params.generations as usize);

        for attempt in 0..num_runs {
            self.init_pop(&params);

            let mut best_seen = self.select_best().fitness;
            let mut stagnation_for_injection = 0usize;

            for generation in 0..params.generations {
                let (selected_parents, offspring) = self.create_offspring(&params);
                self.select_survivors(selected_parents, offspring, &params);

                let current_best = self.select_best().fitness;

                // Keep track of stagnation
                if current_best + 1e-3 < best_seen {
                    best_seen = current_best;
                    stagnation_for_injection = 0;
                } else {
                    stagnation_for_injection += 1;
                }

                if params.stagnation_for_injection > 0
                    && stagnation_for_injection >= params.stagnation_for_injection
                {
                    self.inject_diversity(attempt, generation, &params);
                    stagnation_for_injection = 0;
                }

                // Record population stats
                let (min_fitness, mean_fitness, max_fitness) =
                    population_fitness_stats(&self.population);
                let entropy = population_route_edge_entropy(&self.population);
                fitness_history.push(FitnessHistoryRow {
                    run: attempt + 1,
                    generation: generation + 1,
                    min_fitness,
                    mean_fitness,
                    max_fitness,
                    entropy,
                });

                // Print progression
                let progress =
                    format_progress_bar(generation + 1, params.generations, PROGRESS_BAR_WIDTH);
                print!(
                    "\rRun {}/{} {} best={:.3}",
                    attempt + 1,
                    num_runs,
                    progress,
                    self.select_best().fitness
                );
                flush_stdout();
            }

            let best_attempt = self.select_best().clone();
            if best_global
                .as_ref()
                .map_or(true, |best| best_attempt.fitness < best.fitness)
            {
                best_global = Some(best_attempt);
            }

            println!();
            println!(
                "[Run] Run {}/{} best so far: {:.4}",
                attempt + 1,
                num_runs,
                best_global.as_ref().unwrap().fitness
            );
        }

        let best_solution = best_global.unwrap_or_else(|| self.select_best().clone());

        // Save results
        let history_path = "fitness_history.csv";
        if let Err(err) = write_fitness_history_csv(history_path, &fitness_history) {
            eprintln!(
                "[Run] Failed to write fitness history to {}: {}",
                history_path, err
            );
        } else {
            println!("[Run] Fitness history saved to {}", history_path);
        }

        export(self, &best_solution).expect("[Error] Failed to export as json");
        println!("[Run] Final best fitness: {:.4}", best_solution.fitness);
    }
}
