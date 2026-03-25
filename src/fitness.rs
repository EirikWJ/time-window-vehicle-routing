use crate::instance::Instance;
use crate::individual::Individual;
use crate::local_search::{CHILD_LOCAL_SEARCH_PASSES, improve_solution};
use crate::ga_params::GaParams;

const CAPACITY_PENALTY: f32 = 50.0;
const TIME_WINDOW_PENALTY: f32 = 50.0;
const RETURN_TIME_PENALTY: f32 = 50.0;
const INFEASIBLE_PENALTY: f32 = 1.0e12;

impl Instance{

    // calculates the fitness based on the defined constrains
    /*
    Contraints:
        Capacity_nurse: the load a patient has on a nurses already existing load musn't be above the total capacity
        Start_time: patient care must be done after this time
        End_time: patient care must be completed before this time.
        Depot_return_time: if the nurse isnt back to the depot before end of day. Blåtimen er over!!! :(

    Infeasable routes:
        A ridicously high penalty is given if:
            if there hasnt been defined any routes. what the helly??
            there has been defined a number of routes that exeeds the amount of available nurses.
            all the routes doesn't visist all patients. 
    */
    pub fn calc_fitness( &self, 
        genotype: &[u32], 
        routes: &[(usize, usize)]
    ) -> f32 {
        if routes.is_empty() { return INFEASIBLE_PENALTY } // no routes defined. ermmm
        if routes.len() > self.nbr_nurses as usize { return INFEASIBLE_PENALTY } // more routes than nurses
        let assigned: usize = routes.iter().map(|(s, e)| e - s + 1).sum();
        if assigned != genotype.len() { return INFEASIBLE_PENALTY } // not all patients have been visited

        let mut total_cost: f32 = 0.0;

        for &(s, e) in routes {
            let route: &[u32] = &genotype[s..=e];
            if route.is_empty() { continue } // a shift consists of sitting in the break room drinking coffee and eating doughnuts 

            let mut time: f32 = 0.0;
            let mut load: u32 = 0;
            let mut prev_node: usize = 0;

            for &patient_id in route {
                let node: usize = patient_id as usize;
                let travel: f32 = self.travel_times[prev_node][node] as f32;

                time += travel;
                total_cost += travel;

                let patient = &self.patients[&patient_id.to_string()];

                load += patient.demand;
                if load > self.capacity_nurse { // penalty if the nurse is overworked
                    total_cost += CAPACITY_PENALTY * (load - self.capacity_nurse) as f32;
                }

                if time < patient.start_time as f32 { // penalty if nurse is early to an appointment
                    time = patient.start_time as f32;
                }

                let service_end = time + patient.care_time as f32;
                if service_end > patient.end_time as f32 { // penalty if the nurse has overstayed their welcome
                    total_cost += TIME_WINDOW_PENALTY * (service_end - patient.end_time as f32);
                }

                time = service_end;
                prev_node = node;
            }

            let return_travel = self.travel_times[prev_node][0] as f32;
            time += return_travel;
            total_cost += return_travel;

            if time > self.depot.return_time as f32 { // blåtimen er over
                total_cost += RETURN_TIME_PENALTY * (time - self.depot.return_time as f32);
            }
        }

        total_cost
    }
    // calculates the fitness of an individual based on its genotype
    pub fn evaluate_child( &self, 
        genotype: Vec<u32>, 
        params: &GaParams
    ) -> Individual {
        let routes = self.decode_routes_from_permutation(&genotype);
        let fitness = self.calc_fitness(&genotype, &routes);
        let mut child = Individual {
            genotype,
            fitness,
            routes,
        };
        // if the fitness exists and isnt stupidly high do local search to improve the fitness :3
        if child.fitness.is_finite() && child.fitness < params.feasible_fitness_threshold {
            improve_solution(self, &mut child, CHILD_LOCAL_SEARCH_PASSES);
        }
        child
    }

    pub fn select_best(&self) -> &Individual {
        self.population
            .iter()
            .min_by(|a, b| a.fitness.total_cmp(&b.fitness))
            .expect("Empty population!")
    }
}