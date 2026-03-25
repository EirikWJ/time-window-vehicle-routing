#[derive(Debug, Default, Clone)]
pub struct Individual {
    pub genotype: Vec<u32>,
    pub fitness: f32,
    pub routes: Vec<(usize, usize)>,
}
