use std::fs::File;
use std::io::BufReader;

use crate::instance::Instance;

// reads the provided json files and fills them into the problem instance.
pub fn load_instance(path: &str) -> Result<Instance, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let instance: Instance = serde_json::from_reader(reader)?;
    println!("{}", instance.instance_name);
    println!("Benchmark    5%             10%           20%           30%");
    println!(
        "{}      {}      {}      {}        {}",
        instance.benchmark,
        instance.benchmark * 1.05,
        instance.benchmark * 1.1,
        instance.benchmark * 1.2,
        instance.benchmark * 1.3
    );
    Ok(instance)
}
