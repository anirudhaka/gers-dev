// Define the Genome type as a vector of integers
pub type Genome = Vec<usize>;

// Constants for genome parameters
const MAX_GENOME_LENGTH: usize = 100;
const MAX_GENE_VALUE: usize = 255;

// Function to initialize a random genome
pub fn initialize_genome() -> Genome {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let length = rng.gen_range(1..=MAX_GENOME_LENGTH);
    (0..length).map(|_| rng.gen_range(0..=MAX_GENE_VALUE)).collect()
}


// Function to extend a genome
pub fn extend_genome(genome: &mut Genome, extension_length: usize) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    for _ in 0..extension_length {
        genome.push(rng.gen_range(0..=MAX_GENE_VALUE));
    }
}

// Function to truncate a genome
pub fn truncate_genome(genome: &mut Genome, truncation_length: usize) {
    for _ in 0..truncation_length {
        genome.pop();
    }
}