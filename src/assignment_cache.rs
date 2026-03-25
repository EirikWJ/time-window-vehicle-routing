use crate::individual::Individual;

#[derive(Clone)]
pub struct AssignmentCache {
    pub assign: Vec<u16>, // index by patient id, index 0 is unused
}

/// Find the lowest patient ID in a route
fn route_min_patient_inclusive(genotype: &[u32], s: usize, e: usize) -> u32 {
    let mut m = genotype[s];
    for &p in &genotype[s..=e] {
        if p < m {
            m = p;
        }
    }
    m
}

/// Build the assignment cache used for comparing individuals in crowding
pub fn build_assignment_cache(ind: &Individual, n_patients: usize) -> AssignmentCache {
    let mut assign = vec![u16::MAX; n_patients + 1];
    let routes = &ind.routes;

    let mut route_idx: Vec<usize> = (0..routes.len()).collect();

    // sort route order independent of nurse index, sorted by lowest patient id in route
    route_idx.sort_unstable_by_key(|&ri| {
        let (s, e) = routes[ri];
        route_min_patient_inclusive(&ind.genotype, s, e)
    });

    // fill the cache
    for (canon_route, &ri) in route_idx.iter().enumerate() {
        let (s, e) = routes[ri];

        for &pid in &ind.genotype[s..=e] {
            let p = pid as usize;
            assign[p] = canon_route as u16;
        }
    }

    AssignmentCache { assign }
}

/// Compare the assignment cache of two individuals
pub fn assignment_distance(a: &AssignmentCache, b: &AssignmentCache, n_patients: usize) -> f32 {
    let mut same = 0;
    for p in 1..=n_patients {
        same += (a.assign[p] == b.assign[p]) as usize;
    }
    1.0 - (same as f32 / n_patients as f32)
}
