use crate::individual::Individual;
use crate::instance::Instance;
use crate::utils::{pack_routes, unpack_routes};

pub const INIT_LOCAL_SEARCH_PASSES: usize = 8;
pub const CHILD_LOCAL_SEARCH_PASSES: usize = 4;
pub const INJECTION_LOCAL_SEARCH_PASSES: usize = 5;

const IMPROVEMENT_EPSILON: f32 = 1e-4;

fn route_distance(
    instance: &Instance, 
    route: &[u32]
) -> f32 {
    if route.is_empty() { return 0.0; }

    let mut total = 0.0f32;
    let mut prev = 0usize;

    for &pid in route {
        let node = pid as usize;
        total += instance.travel_times[prev][node] as f32;
        prev = node;
    }

    total + instance.travel_times[prev][0] as f32
}

/// Tries to improve an individual by reversing every possible segment in every route
fn try_best_two_opt(
    instance: &Instance, 
    routes: &mut [Vec<u32>]
) -> bool {
    let mut best_improvement = 0.0f32;
    let mut best_move: Option<(usize, usize, usize)> = None;

    for (route_idx, route) in routes.iter().enumerate() {
        if route.len() < 4 {
            continue;
        }

        let old_dist = route_distance(instance, route);

        // Try to reverse every segment
        for i in 0..(route.len() - 2) {
            for j in (i + 1)..(route.len() - 1) {
                let mut candidate = route.clone();
                candidate[i..=j].reverse();

                // Skip infeasible routes
                if !instance.is_route_feasible(&candidate) {
                    continue;
                }

                // Compare old route distance to reversed route distance
                let new_dist = route_distance(instance, &candidate);
                let improvement = old_dist - new_dist;

                if improvement > best_improvement + IMPROVEMENT_EPSILON {
                    best_improvement = improvement;
                    best_move = Some((route_idx, i, j));
                }
            }
        }
    }

    // Replace existing route with improved route, if one exists
    if let Some((route_idx, i, j)) = best_move {
        routes[route_idx][i..=j].reverse();
        return true;
    }

    false
}

/// Tries to improve an individual by removing patients from routes and trying to add them to every position in every other route
fn try_best_relocate(
    instance: &Instance, 
    routes: &mut Vec<Vec<u32>>
) -> bool {
    let mut best_delta = 0.0f32;
    let mut best_move: Option<(usize, usize, usize, usize)> = None;

    let old_len = routes.len();
    let allow_new_route = old_len < instance.nbr_nurses as usize;

    // Go through every route to remove patients
    for src_idx in 0..old_len {
        if routes[src_idx].is_empty() {
            continue;
        }

        let src_old = routes[src_idx].clone();
        let src_old_cost = route_distance(instance, &src_old);

        // Remove patient from route
        for src_pos in 0..src_old.len() {
            let pid = src_old[src_pos];

            let mut src_new = src_old.clone();
            src_new.remove(src_pos);

            // Skip infeasible routes
            let src_new_cost = if src_new.is_empty() {
                0.0
            } else {
                if !instance.is_route_feasible(&src_new) {
                    continue;
                }
                route_distance(instance, &src_new)
            };

            // Make it possible to add patient to new route if number of routes is not exceeded
            let dst_upper = if allow_new_route {
                old_len + 1
            } else {
                old_len
            };

            // Go through every route except the current one to insert
            for dst_idx in 0..dst_upper {
                if dst_idx == src_idx {
                    continue;
                }

                // Decide if route to insert into is existing or new
                let dst_old = if dst_idx < old_len {
                    routes[dst_idx].clone()
                } else {
                    Vec::new()
                };

                let dst_old_cost = route_distance(instance, &dst_old);

                // Try every position in route to insert
                for ins_pos in 0..=dst_old.len() {
                    let mut dst_new = dst_old.clone();
                    dst_new.insert(ins_pos, pid);

                    // Skip infeasible routes
                    if !instance.is_route_feasible(&dst_new) {
                        continue;
                    }

                    // Compare with current best insertion
                    let dst_new_cost = route_distance(instance, &dst_new);
                    let delta = (src_new_cost + dst_new_cost) - (src_old_cost + dst_old_cost);

                    // Keep best
                    if delta < best_delta - IMPROVEMENT_EPSILON {
                        best_delta = delta;
                        best_move = Some((src_idx, src_pos, dst_idx, ins_pos));
                    }
                }
            }
        }
    }

    let Some((src_idx, src_pos, dst_idx, ins_pos)) = best_move else {
        return false;
    };

    // Remove patient from source route
    let prev_len = routes.len();
    let pid = routes[src_idx].remove(src_pos);
    let src_became_empty = routes[src_idx].is_empty();

    if src_became_empty {
        routes.remove(src_idx);
    }

    // If source route disappears, right-hand indices shift by one.
    let actual_dst = if dst_idx == prev_len {
        routes.len()
    } else if src_became_empty && dst_idx > src_idx {
        dst_idx - 1
    } else {
        dst_idx
    };

    // Insert patient into better route
    if actual_dst == routes.len() {
        routes.push(vec![pid]);
    } else {
        routes[actual_dst].insert(ins_pos, pid);
    }

    true
}

/// Run local search `max_passes` times to improve an individual
/// Will only run several times if improvements have been made
/// No improvement = break
pub fn improve_solution(
    instance: &Instance, 
    individual: &mut Individual, 
    max_passes: usize
) {
    if max_passes == 0 {
        return;
    }

    let mut routes = unpack_routes(&individual.genotype, &individual.routes);
    routes.retain(|route| !route.is_empty());

    for _ in 0..max_passes {
        let mut improved = false;

        if try_best_relocate(instance, &mut routes) {
            improved = true;
        }

        if try_best_two_opt(instance, &mut routes) {
            improved = true;
        }

        if !improved {
            break;
        }
    }

    let (genotype, packed) = pack_routes(&routes);
    individual.fitness = instance.calc_fitness(&genotype, &packed);
    individual.genotype = genotype;
    individual.routes = packed;
}
