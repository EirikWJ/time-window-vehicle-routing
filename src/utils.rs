use rand::Rng;
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::individual::Individual;

#[derive(Debug, Deserialize, Clone)]
pub struct Coordinate {
    #[serde(rename = "x_coord", deserialize_with = "deserialize_i32_from_number")]
    pub x: i32,

    #[serde(rename = "y_coord", deserialize_with = "deserialize_i32_from_number")]
    pub y: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct FitnessHistoryRow {
    pub run: usize,
    pub generation: u32,
    pub min_fitness: f32,
    pub mean_fitness: f32,
    pub max_fitness: f32,
    pub entropy: f32,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberRepr {
    Int(i64),
    Float(f64),
}

pub fn get_2idx(n: usize) -> (usize, usize) {
    let mut rng = rand::thread_rng();
    let ia = rng.gen_range(0..n - 1);
    let ib = rng.gen_range(ia + 1..n);
    (ia, ib)
}

/// Convert route spans (start and end for every route) for genotype into list of routes 
pub fn unpack_routes(genotype: &[u32], route_spans: &[(usize, usize)]) -> Vec<Vec<u32>> {
    let mut routes = Vec::with_capacity(route_spans.len());
    for &(start, end) in route_spans {
        routes.push(genotype[start..=end].to_vec());
    }
    routes
}

/// Convert list of routes into list of tuples containing start and end of every route in genotype
pub fn pack_routes(routes: &[Vec<u32>]) -> (Vec<u32>, Vec<(usize, usize)>) {
    let mut genotype = Vec::new();
    let mut packed = Vec::new();

    for route in routes {
        if route.is_empty() { continue; }

        let start = genotype.len();
        genotype.extend_from_slice(route);
        let end = genotype.len() - 1;
        packed.push((start, end));
    }

    (genotype, packed)
}

/// Compute the min, mean, and max fitness for a population
pub fn population_fitness_stats(population: &[Individual]) -> (f32, f32, f32) {
    if population.is_empty() { return (f32::NAN, f32::NAN, f32::NAN); }

    let mut min_fitness = f32::INFINITY;
    let mut max_fitness = f32::NEG_INFINITY;
    let mut total = 0.0f32;

    for individual in population {
        let fitness = individual.fitness;
        if fitness < min_fitness {
            min_fitness = fitness;
        }
        if fitness > max_fitness {
            max_fitness = fitness;
        }
        total += fitness;
    }

    let mean_fitness = total / population.len() as f32;
    (min_fitness, mean_fitness, max_fitness)
}

/// Create edges for every connecting patient in a rout (depot included in start and end)
fn collect_individual_route_edges(individual: &Individual) -> HashSet<(u32, u32)> {
    let mut edges = HashSet::new();

    for &(start, end) in &individual.routes {
        if start > end || end >= individual.genotype.len() { continue; }

        let route = &individual.genotype[start..=end];
        if route.is_empty() { continue; }

        // Depot to first patient
        edges.insert((0, route[0]));

        // Edges between consecutive patients
        for pair in route.windows(2) {
            edges.insert((pair[0], pair[1]));
        }

        // Last patient to depot
        edges.insert((route[route.len() - 1], 0));
    }

    edges
}

/// Population route-edge presence entropy.
///
/// For each directed edge e, compute fraction of individuals containing e,
/// then use binary entropy h(p) = -p ln p - (1-p) ln(1-p), normalized by ln(2).
/// The final value is the average binary entropy of all edges that appear anywhere in the population
pub fn population_route_edge_entropy(population: &[Individual]) -> f32 {
    if population.is_empty() {
        return f32::NAN;
    }

    let pop_size = population.len();
    if pop_size <= 1 {
        return 0.0;
    }

    // Exctract every edge from every individual and count the times the edges appear
    let mut edge_presence_counts: HashMap<(u32, u32), usize> = HashMap::new();
    for individual in population {
        let edges_in_individual = collect_individual_route_edges(individual);
        for edge in edges_in_individual {
            *edge_presence_counts.entry(edge).or_insert(0) += 1;
        }
    }

    if edge_presence_counts.is_empty() { return 0.0; }

    let pop_size_f = pop_size as f32;
    let ln2 = 2.0f32.ln();
    let mut entropy_sum = 0.0f32;
    
    // Calculate entropy
    for count in edge_presence_counts.values() {
        let p = *count as f32 / pop_size_f;
        if p <= 0.0 || p >= 1.0 { continue; }

        let binary_entropy = -(p * p.ln()) - ((1.0 - p) * (1.0 - p).ln());
        entropy_sum += binary_entropy / ln2;
    }

    entropy_sum / edge_presence_counts.len() as f32
}

pub fn write_fitness_history_csv<P: AsRef<Path>>(
    path: P,
    rows: &[FitnessHistoryRow],
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "run,generation,min_fitness,mean_fitness,max_fitness,entropy"
    )?;

    for row in rows {
        writeln!(
            file,
            "{},{},{:.6},{:.6},{:.6},{:.6}",
            row.run,
            row.generation,
            row.min_fitness,
            row.mean_fitness,
            row.max_fitness,
            row.entropy
        )?;
    }

    Ok(())
}

pub fn deserialize_u32_from_number<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = NumberRepr::deserialize(deserializer)?;

    match value {
        NumberRepr::Int(v) => {
            u32::try_from(v).map_err(|_| serde::de::Error::custom("number out of range for u32"))
        }
        NumberRepr::Float(v) => {
            if v.fract() != 0.0 {
                return Err(serde::de::Error::custom("expected whole number for u32"));
            }
            if !(0.0..=(u32::MAX as f64)).contains(&v) {
                return Err(serde::de::Error::custom("number out of range for u32"));
            }
            Ok(v as u32)
        }
    }
}

pub fn deserialize_i32_from_number<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = NumberRepr::deserialize(deserializer)?;

    match value {
        NumberRepr::Int(v) => {
            i32::try_from(v).map_err(|_| serde::de::Error::custom("number out of range for i32"))
        }
        NumberRepr::Float(v) => {
            if v.fract() != 0.0 {
                return Err(serde::de::Error::custom("expected whole number for i32"));
            }
            if v < (i32::MIN as f64) || v > (i32::MAX as f64) {
                return Err(serde::de::Error::custom("number out of range for i32"));
            }
            Ok(v as i32)
        }
    }
}

pub fn format_progress_bar(current: u32, total: u32, width: usize) -> String {
    let safe_total = total.max(1);
    let clamped_current = current.min(safe_total);
    let progress = clamped_current as f32 / safe_total as f32;
    let filled = ((progress * width as f32).round() as usize).min(width);
    let empty = width.saturating_sub(filled);

    format!(
        "[{}{}] {:>3}% ({}/{})",
        "#".repeat(filled),
        " ".repeat(empty),
        (progress * 100.0).round() as u32,
        clamped_current,
        safe_total
    )
}

pub fn flush_stdout() {
    let _ = std::io::stdout().flush();
}
