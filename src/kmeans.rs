use crate::patient::Patient;
use crate::utils::Coordinate;
use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::Array2;
use std::collections::HashMap;

fn coords_to_array(coords: &[Coordinate]) -> Array2<f64> {
    let mut data = Array2::<f64>::zeros((coords.len(), 2));
    for (i, c) in coords.iter().enumerate() {
        data[[i, 0]] = c.x as f64;
        data[[i, 1]] = c.y as f64;
    }
    data
}

/// Assigns each patient to a cluster
pub fn assign_clusters(patients: &HashMap<String, Patient>, n_clusters: usize) -> Vec<Vec<String>> {
    let mut patient_ids: Vec<String> = patients.keys().cloned().collect();
    patient_ids.sort();

    let coords: Vec<Coordinate> = patient_ids.iter().map(|id| patients[id].coord.clone()).collect();

    let dataset = DatasetBase::from(coords_to_array(&coords));
    let model = KMeans::params(n_clusters).fit(&dataset).unwrap();

    let labels = model.predict(&dataset);

    let mut clusters: Vec<Vec<String>> = vec![Vec::new(); n_clusters];

    for (i, id) in patient_ids.iter().enumerate() {
        let cluster_id = labels[i] as usize;
        if cluster_id < n_clusters {
            clusters[cluster_id].push(id.clone());
        }
    }

    clusters
}