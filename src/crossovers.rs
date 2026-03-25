#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::utils::get_2idx;

#[derive(Debug, Clone, Copy)]
pub enum CrossoverType {
    OneOrder,
    PMX,
    EdgeRecombination,
}

// 1 order crossover
/*
1. Choose an arbitrary part from the first parent
2. Copy this part to the first child
3. Copy the numbers that are not in the first part, to the first child:
    - starting right from cut point of the copied part,
    - using the order of the second parent, and
    - wrapping around at the end
4. Analogous for the second child, with parent roles reversed
*/
fn one_order(p1: &mut Vec<u32>, p2: &mut Vec<u32>) -> Vec<u32> {
    let n = p1.len();
    if n < 2 {
        return p1.clone();
    }

    let (ia, ib) = get_2idx(n);
    let mut child: Vec<u32> = vec![u32::MAX; n];
    let mut used: HashSet<u32> = HashSet::with_capacity(n);

    for i in ia..=ib {
        let gene = p1[i];
        child[i] = gene;
        used.insert(gene);
    }

    let mut write_pos = (ib + 1) % n;
    let mut read_pos = (ib + 1) % n;

    while write_pos != ia {
        let gene = p2[read_pos];
        if !used.contains(&gene) {
            child[write_pos] = gene;
            used.insert(gene);
            write_pos = (write_pos + 1) % n;
        }
        read_pos = (read_pos + 1) % n;
    }

    child
}

// PMX
/*
1. Choose random segment and copy it from P1
2. Starting from the first crossover point look for elements
   in that segment of P2 that have not been copied
3. For each of these i look in the offspring to see what
   element j has been copied in its place from P1
4. Place i into the position occupied by j in P2
5. If that position is already filled, keep following mapping chain
6. Fill remaining empty positions from P2
*/
fn pmx(p1: &[u32], p2: &[u32]) -> Vec<u32> {
    let n = p1.len();
    if n < 2 {
        return p1.to_vec();
    }

    let (ia, ib) = get_2idx(n);
    let mut child = vec![u32::MAX; n];

    let mut pos_p2: HashMap<u32, usize> = HashMap::with_capacity(n);
    for (i, &gene) in p2.iter().enumerate() {
        pos_p2.insert(gene, i);
    }

    for i in ia..=ib {
        child[i] = p1[i];
    }

    let segment_genes: HashSet<u32> = child[ia..=ib].iter().copied().collect();

    for i in ia..=ib {
        let gene = p2[i];
        if segment_genes.contains(&gene) {
            continue;
        }

        let mut idx = i;
        loop {
            let mapped = p1[idx];
            idx = *pos_p2
                .get(&mapped)
                .expect("PMX mapping failed: gene from p1 not found in p2");

            if child[idx] == u32::MAX {
                child[idx] = gene;
                break;
            }
        }
    }

    for i in 0..n {
        if child[i] == u32::MAX {
            child[i] = p2[i];
        }
    }

    child
}

// edge recombination
/*
Informal procedure: once edge table is constructed
1. Pick an initial element, entry, at random and put it in the offspring
2. Set current element = entry
3. Remove all references to current element from the table
4. Examine adjacency for current element:
   - If there is a common edge, pick that as next
   - Otherwise pick neighbor with shortest adjacency list
   - Break ties randomly
5. If adjacency is empty, pick random remaining element
*/
fn neighbors(idx: usize, n: usize) -> (usize, usize) {
    let left: usize = if idx == 0 { n - 1 } else { idx - 1 };
    let right: usize = if idx + 1 == n { 0 } else { idx + 1 };
    (left, right)
}

fn add_edge(table: &mut HashMap<u32, HashMap<u32, u8>>, a: u32, b: u32) {
    *table.entry(a).or_default().entry(b).or_insert(0) += 1;
}

fn edge_table(p1: &[u32], p2: &[u32]) -> HashMap<u32, HashMap<u32, u8>> {
    let n = p1.len();
    let mut table: HashMap<u32, HashMap<u32, u8>> = HashMap::new();

    for i in 0..n {
        let curr: u32 = p1[i];
        let (l1, r1) = neighbors(i, n);
        add_edge(&mut table, curr, p1[l1]);
        add_edge(&mut table, curr, p1[r1]);
    }

    for i in 0..n {
        let curr = p2[i];
        let (l2, r2) = neighbors(i, n);
        add_edge(&mut table, curr, p2[l2]);
        add_edge(&mut table, curr, p2[r2]);
    }

    table
}

fn pick_next(curr_neighbors: &HashMap<u32, u8>, table: &HashMap<u32, HashMap<u32, u8>>) -> u32 {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    let common: Vec<u32> = curr_neighbors
        .iter()
        .filter(|&(_, &count)| count == 2)
        .map(|(&n, _)| n)
        .collect();

    let candidates: Vec<u32> = if !common.is_empty() {
        common
    } else {
        curr_neighbors.keys().cloned().collect()
    };

    if !candidates.is_empty() {
        let min_size: usize = candidates
            .iter()
            .map(|n| table.get(n).map_or(usize::MAX, |m| m.len()))
            .min()
            .unwrap();

        let best: Vec<u32> = candidates
            .into_iter()
            .filter(|n| table.get(n).map_or(usize::MAX, |m| m.len()) == min_size)
            .collect();

        *best.choose(&mut rng).unwrap()
    } else {
        *table
            .keys()
            .cloned()
            .collect::<Vec<u32>>()
            .choose(&mut rng)
            .unwrap()
    }
}

fn edge_recombination(p1: &[u32], p2: &[u32]) -> Vec<u32> {
    use rand::Rng;
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

    let mut table: HashMap<u32, HashMap<u32, u8>> = edge_table(p1, p2);
    let mut child: Vec<u32> = Vec::with_capacity(p1.len());

    let mut curr_elem = p1[rng.gen_range(0..p1.len())];

    while !table.is_empty() {
        child.push(curr_elem);

        let neighbors: HashMap<u32, u8> = table.get(&curr_elem).cloned().unwrap_or_default();

        for map in table.values_mut() {
            map.remove(&curr_elem);
        }

        table.remove(&curr_elem);

        if table.is_empty() {
            break;
        }

        curr_elem = pick_next(&neighbors, &table);
    }

    child
}

// crossover(&mut arr1, &mut arr2, 1.0, CrossoverType::OneOrder)
pub fn crossover(
    p1: &mut Vec<u32>,
    p2: &mut Vec<u32>,
    rate: f32,
    method: CrossoverType,
) -> (Vec<u32>, Vec<u32>) {
    if rand::random::<f32>() > rate {
        return (p1.clone(), p2.clone());
    }

    match method {
        CrossoverType::OneOrder => (one_order(p1, p2), one_order(p2, p1)),
        CrossoverType::PMX => (pmx(p1, p2), pmx(p2, p1)),
        CrossoverType::EdgeRecombination => {
            (edge_recombination(p1, p2), edge_recombination(p2, p1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn assert_valid_permutation(child: &[u32], parent: &[u32]) {
        assert_eq!(
            child.len(),
            parent.len(),
            "child length differs from parent"
        );

        let child_set: HashSet<u32> = child.iter().copied().collect();
        let parent_set: HashSet<u32> = parent.iter().copied().collect();

        assert_eq!(child_set.len(), parent.len(), "child has duplicates");
        assert_eq!(
            child_set, parent_set,
            "child is not a permutation of parent"
        );
    }

    #[test]
    fn one_order_handles_one_based_ids() {
        let mut p1: Vec<u32> = (1..=93).collect();
        let mut p2: Vec<u32> = p1.clone();
        p2.reverse();

        let child: Vec<u32> = one_order(&mut p1, &mut p2);
        assert_valid_permutation(&child, &p1);
    }

    #[test]
    fn pmx_handles_one_based_ids() {
        let p1: Vec<u32> = (1..=93).collect();
        let mut p2: Vec<u32> = p1.clone();
        p2.reverse();

        let child: Vec<u32> = pmx(&p1, &p2);
        assert_valid_permutation(&child, &p1);
    }
}
