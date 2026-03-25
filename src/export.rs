use serde_json::{Map, Value, json};
use std::error::Error;
use std::fmt::Write as FmtWrite;

use crate::{individual::Individual, instance::Instance};

struct RouteLineData {
    duration: f32,
    covered_demand: u32,
    sequence: String,
} // the specified output in a struct for organisation

// fills a nurse line for the txt output
fn build_route_line(instance: &Instance, route: &[u32]) -> RouteLineData {
    if route.is_empty() { // default for empty routes
        return RouteLineData {
            duration: 0.0,
            covered_demand: 0,
            sequence: "D (0.00) -> D (0.00)".to_string(),
        };
    }

    let mut time = 0.0f32;
    let mut covered_demand = 0u32;
    let mut prev_node = 0usize;
    let mut parts = vec![format!("D ({:.2})", time)];

    for &pid in route {
        let node = pid as usize;
        let patient = &instance.patients[&pid.to_string()];

        let service_start = (time + instance.travel_times[prev_node][node] as f32).max(patient.start_time as f32);
        let service_end = service_start + patient.care_time as f32;

        covered_demand += patient.demand;
        parts.push(format!( // for each paptient, [time period for nurse visit] [allowed visitation time]
            "{} ({:.2}-{:.2}) [{}-{}]",
            pid, service_start, service_end, patient.start_time, patient.end_time
        ));

        time = service_end;
        prev_node = node;
    }

    time += instance.travel_times[prev_node][0] as f32;
    parts.push(format!("D ({:.2})", time));

    RouteLineData {
        duration: time,
        covered_demand,
        sequence: parts.join(" -> "),
    }
}

// fancy schmancy output as asked for 
fn export_txt(instance: &Instance, individual: &Individual) -> Result<(), Box<dyn Error>> {
    let mut output = String::new();

    writeln!(&mut output, "Nurse capacity: {}", instance.capacity_nurse)?;
    writeln!(&mut output)?;
    writeln!(&mut output,"Depot return time: {}",instance.depot.return_time)?;
    writeln!(&mut output,"--------------------------------------------------------------------------")?;
    // write each line for each of the nurses
    for nurse_idx in 0..instance.nbr_nurses as usize {
        // group routes for each nurse together
        let route = if nurse_idx < individual.routes.len() {
            let (s, e) = individual.routes[nurse_idx];
            individual.genotype[s..=e].to_vec()
        } else {
            Vec::new()
        };

        let line = build_route_line(instance, &route);
        writeln!(
            &mut output,
            "Nurse {:<2} {:>10.2} {:>10}    {}",
            nurse_idx + 1,
            line.duration,
            line.covered_demand,
            line.sequence
        )?;
    }
    writeln!(&mut output,"--------------------------------------------------------------------------")?;
    writeln!(&mut output,"Objective value (total duration): {:.2}",individual.fitness)?;

    std::fs::write("solution.txt", output)?;
    println!("[Export] Best solution written to solution.txt");

    Ok(())
}

// export individual to route visualisation
pub fn export(instance: &Instance, individual: &Individual) -> Result<(), Box<dyn Error>> {
    let mut patients_json = Map::new();

    for (id, p) in &instance.patients {
        patients_json.insert(
            id.clone(),
            json!({
                "x": p.coord.x,
                "y": p.coord.y
            }),
        );
    }

    let mut routes_json = Vec::new();

    for (s, e) in &individual.routes {
        let route: Vec<u32> = individual.genotype[*s..=*e].to_vec();
        routes_json.push(route);
    }

    let data = json!({
        "depot": {
            "x": instance.depot.x_coord,
            "y": instance.depot.y_coord
        },
        "patients": Value::Object(patients_json),
        "routes": routes_json
    });

    std::fs::write("individual.json", serde_json::to_string_pretty(&data)?)?;
    export_txt(instance, individual)?;

    println!("[Export] Exported individual with fitness {} as JSON",individual.fitness);

    Ok(())
}