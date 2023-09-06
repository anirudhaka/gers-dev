use rand::Rng;
use crate::evolutionary_ops::{mutate, tournament_selection, one_point_crossover};

type Genome = Vec<usize>;
type Population = Vec<Genome>;

const POPULATION_SIZE: usize = 100;
// const MAX_GENERATIONS: usize = 1000;
const ELITISM_COUNT: usize = 5;

// Initialize a random population
pub fn initialize_population(size: usize) -> Population {
    (0..size).map(|_| {
        let mut rng = rand::thread_rng();
        (0..rng.gen_range(1..100)).map(|_| rng.gen_range(0..256)).collect()
    }).collect()
}

pub fn random_initialization(pop_size: usize, genome_length: usize, max_gene_value: usize) -> Population {
    let mut rng = rand::thread_rng();
    let mut population: Vec<Vec<usize>> = Vec::with_capacity(POPULATION_SIZE);

    for _ in 0..pop_size {
        let individual: Vec<usize> = (0..genome_length)
            .map(|_| rng.gen_range(0..max_gene_value))
            .collect();
        population.push(individual);
    }

    population
}

// Evolve the population for one generation
pub fn evolve_population(population: &Population, fitness: &dyn Fn(&Vec<usize>) -> usize) -> Population {
    let mut new_population = Vec::with_capacity(POPULATION_SIZE);

    // Sort by fitness
    let mut sorted_population = population.clone();
    sorted_population.sort_by_key(|genome| fitness(genome));

    // Elitism: directly carry over the best genomes
    for i in 0..ELITISM_COUNT {
        new_population.push(sorted_population[i].clone());
    }

    // Rest of the new population is filled by offspring from crossover and mutation
    while new_population.len() < POPULATION_SIZE {
        let parent1 = tournament_selection(&sorted_population, 2);
        let parent2 = tournament_selection(&sorted_population, 2);
        let (mut child1, mut child2) = one_point_crossover(parent1, parent2);

        mutate(&mut child1);
        mutate(&mut child2);

        new_population.push(child1);
        new_population.push(child2);
    }

    new_population
}