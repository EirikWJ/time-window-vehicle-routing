use crate::{individual::Individual, instance::Instance};
use rand::Rng;
use rand::seq::SliceRandom;

impl Instance {
    /// Checks if a given route is valid, following the given constraints
    pub fn is_route_feasible(&self, route: &[u32]) -> bool {
        let mut time: f32 = 0.0;
        let mut load: u32 = 0;
        let mut prev_node: usize = 0;

        for &pid in route {
            let node = pid as usize;
            time += self.travel_times[prev_node][node] as f32;

            let patient = &self.patients[&pid.to_string()];

            // Capacity constraint
            load += patient.demand;
            if load > self.capacity_nurse {
                return false;
            }

            // Wait until patient start time
            if time < patient.start_time as f32 {
                time = patient.start_time as f32;
            }

            // Patient care window constraint (service must finish before end)
            let service_end = time + patient.care_time as f32;
            if service_end > patient.end_time as f32 {
                return false;
            }

            time = service_end;
            prev_node = node;
        }

        // Depot return time constraint
        time += self.travel_times[prev_node][0] as f32;
        time <= self.depot.return_time as f32
    }

    /// Tries to insert a patient into a route, checking a limited amount of positions
    pub fn find_best_insertion_limited(
        &self,
        route: &[u32],
        pid: u32,
        rng: &mut impl Rng,
        extra_positions: usize,
    ) -> Option<(usize, f32)> {
        let route_length: usize = route.len();
        let mut positions = Vec::with_capacity(3 + extra_positions);

        // Will check positions: first, middle, and last in route
        positions.push(0);
        if route_length > 0 {
            positions.push(route_length / 2);
        }
        positions.push(route_length);

        // Will also check `extra_positions` number of random places
        for _ in 0..extra_positions {
            let pos = rng.gen_range(0..=route_length);
            positions.push(pos);
        }

        positions.sort_unstable();
        positions.dedup();

        let mut best: Option<(usize, f32)> = None;

        for pos in positions {
            // Decide where the new patient will go
            let prev_node = if pos == 0 { 0 } else { route[pos - 1] as usize };
            let next_node = if pos == route_length {
                0
            } else {
                route[pos] as usize
            };
            let node = pid as usize;

            // Added travel cost of inserting node between prev and next node
            let delta = (self.travel_times[prev_node][node] + self.travel_times[node][next_node]
                - self.travel_times[prev_node][next_node]) as f32;

            // Build candidate route
            let mut candidate_route = Vec::with_capacity(route_length + 1);
            candidate_route.extend_from_slice(&route[..pos]);
            candidate_route.push(pid);
            candidate_route.extend_from_slice(&route[pos..]);

            // Choose the new route with the least added travel time (among feasible routes)
            if self.is_route_feasible(&candidate_route) {
                if best.map_or(true, |(_, best_delta)| delta < best_delta) {
                    best = Some((pos, delta));
                }
            }
        }

        best
    }

    /// Tries to construct a single feasible individual
    pub fn init_feasible_individual(
        &self,
        rng: &mut impl Rng,
        max_restarts: usize,
        seed_top_k: usize,
        candidate_sample: usize,
        extra_positions: usize,
    ) -> Option<Individual> {
        let num_nurses = self.nbr_nurses as usize;

        let all: Vec<u32> = self
            .patients
            .keys()
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();

        // Create feasible individual, repeat up to `max_restarts` times if construction fails
        for _ in 0..max_restarts {
            let mut unassigned = all.clone();

            // Sort by end_time to assign earliest patients first
            unassigned.sort_unstable_by_key(|&id| self.patients[&id.to_string()].end_time);

            let mut routes: Vec<Vec<u32>> = Vec::new();

            // Build routes until all patients are assigned to a route
            while !unassigned.is_empty() {
                // Failed if all nurses are used up already, try again
                if routes.len() >= num_nurses {
                    routes.clear();
                    break;
                }

                // Pick first patient randomly from the k earliest patients (based on end time)
                let k = seed_top_k.min(unassigned.len()).max(1);
                let seed_idx = rng.gen_range(0..k);
                let seed = unassigned.remove(seed_idx);

                let mut route = vec![seed];
                if !self.is_route_feasible(&route) {
                    routes.clear();
                    break;
                }

                // Create current route
                loop {
                    // Done if there was only one unassigned patient left
                    if unassigned.is_empty() {
                        break;
                    }

                    // Sample candidates randomly
                    let mut cand_ids: Vec<u32> = if unassigned.len() <= candidate_sample {
                        unassigned.clone()
                    } else {
                        let mut tmp = unassigned.clone();
                        tmp.shuffle(rng);
                        tmp.truncate(candidate_sample);
                        tmp
                    };

                    // Find best feasible insertion among sampled candidates
                    let mut best: Option<(u32, usize, f32)> = None;

                    for pid in cand_ids.drain(..) {
                        if let Some((pos, delta)) =
                            self.find_best_insertion_limited(&route, pid, rng, extra_positions)
                        {
                            if best.map_or(true, |(_, _, best_delta)| delta < best_delta) {
                                best = Some((pid, pos, delta));
                            }
                        }
                    }

                    // If no patients were feasible to add, the route is done
                    let Some((best_pid, best_pos, _)) = best else {
                        break;
                    };

                    // Apply insertion and remove from unassigned
                    route.insert(best_pos, best_pid);
                    if let Some(ix) = unassigned.iter().position(|&x| x == best_pid) {
                        unassigned.swap_remove(ix);
                    }
                }

                routes.push(route);
            }

            // If no routes were created, try again
            if routes.is_empty() {
                continue;
            }

            // Pack routes into genotype + inclusive (s,e)
            let mut genotype = Vec::new();
            let mut packed = Vec::new();

            for r in routes {
                let s = genotype.len();
                genotype.extend_from_slice(&r);
                let e = genotype.len() - 1;
                packed.push((s, e));
            }

            let fitness = self.calc_fitness(&genotype, &packed);
            return Some(Individual {
                genotype,
                fitness,
                routes: packed,
            });
        }

        None
    }

    /// Create routes from a genotype.
    /// Greedily add the next patient to the current nurse until capacity is reached, then start new route
    pub fn decode_routes_from_permutation(&self, genotype: &[u32]) -> Vec<(usize, usize)> {
        let mut routes: Vec<(usize, usize)> = Vec::new();

        if genotype.is_empty() {
            return routes;
        }

        let mut start = 0usize;
        let mut current_route: Vec<u32> = Vec::new();
        
        // Add patient to route only if feasible, if not, start new route
        for (i, &pid) in genotype.iter().enumerate() {
            current_route.push(pid);

            if !self.is_route_feasible(&current_route) {
                current_route.pop();

                if !current_route.is_empty() {
                    routes.push((start, i - 1));
                }

                if routes.len() >= self.nbr_nurses as usize {
                    return Vec::new();
                }

                current_route.clear();
                current_route.push(pid);
                start = i;

                // Single patient cannot be served: infeasible chromosome
                if !self.is_route_feasible(&current_route) {
                    return Vec::new();
                }
            }
        }

        if !current_route.is_empty() {
            routes.push((start, genotype.len() - 1));
        }

        if routes.len() > self.nbr_nurses as usize {
            return Vec::new();
        }

        routes
    }
}
