use rand::Rng;

type Genome = Vec<usize>;
type Population = Vec<Genome>;

// Tournament Selection
pub fn tournament_selection(population: &Population, tournament_size: usize) -> &Genome {
    let mut rng = rand::thread_rng();
    let mut best = &population[rng.gen_range(0..population.len())];

    for _ in 1..tournament_size {
        let contender = &population[rng.gen_range(0..population.len())];
        // Here the assumption is that lower genome length has better fitness.
        // TODO: use a fitness function.
        if contender.len() < best.len() {
            best = contender;
        }
    }

    best
}

// One-point Crossover
pub fn one_point_crossover(parent1: &Genome, parent2: &Genome) -> (Genome, Genome) {
    let mut rng = rand::thread_rng();
    let crossover_point = rng.gen_range(0..parent1.len().min(parent2.len()));

    let child1: Genome = parent1[..crossover_point].iter().chain(&parent2[crossover_point..]).cloned().collect();
    let child2: Genome = parent2[..crossover_point].iter().chain(&parent1[crossover_point..]).cloned().collect();

    (child1, child2)
}

// Mutation
pub fn mutate(genome: &mut Genome) {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..genome.len());
    genome[index] = rng.gen_range(0..256);
}