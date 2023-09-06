use rand::Rng;
use std::collections::HashMap;
use lazy_static::lazy_static;
use std::f64;

type Genome = Vec<u8>;

const POPULATION_SIZE: usize = 10;
const MUTATION_RATE: f64 = 0.01;

const MAX_GENERATIONS: usize = 10;


lazy_static! {
    static ref GRAMMAR: HashMap<&'static str, Vec<&'static str>> = {
        let mut map = HashMap::new();
        map.insert("S", vec!["E"]);
        map.insert("E", vec!["E + T", "E - T", "T"]);
        map.insert("T", vec!["T * F", "T / F", "F"]);
        map.insert("F", vec!["x", "y", "( E )", "1.0", "2.0", "3.0"]);
        map
    };
}


fn evaluate_fitness(expression: &str, data: &[(f64, f64, f64)]) -> f64 {

    fn calculate_mse(expression: &str, data: &[(f64, f64, f64)]) -> f64 {
        let mut total_error = 0.0;

        for (x, y, actual_value) in data {
            // println!("x: {} y: {}", x, y);
            let predicted_value = evaluate_expression(expression, *x, *y);
            let error = predicted_value - actual_value;
            total_error += error * error;
        }

        let mse = total_error / data.len() as f64;
        mse
    }

    let mse = calculate_mse(expression, data);
    1.0/(1.0 + mse)
    
}

fn evaluate_expression(expression: &str, x_val: f64, y_val: f64) -> f64 {
    let tokens: Vec<&str> = expression.split_whitespace().collect();

    let mut index = 0;

    fn parse_E(tokens: &[&str], x_val: f64, y_val: f64, index: &mut usize) -> f64 {
        let mut value = parse_T(tokens, x_val, y_val, index);
        while *index < tokens.len() {
            match tokens.get(*index) {
                Some(&"+") => {
                    *index += 1;
                    value += parse_T(tokens, x_val, y_val, index);
                }
                Some(&"-") => {
                    *index += 1;
                    value -= parse_T(tokens, x_val, y_val, index);
                }
                _ => break,
            }
        }
        value
    }

    fn parse_T(tokens: &[&str], x_val: f64, y_val: f64, index: &mut usize) -> f64 {
        let mut value = parse_F(tokens, x_val, y_val, index);
        while *index < tokens.len() {
            match tokens.get(*index) {
                Some(&"*") => {
                    *index += 1;
                    value *= parse_F(tokens, x_val, y_val, index);
                }
                Some(&"/") => {
                    *index += 1;
                    value /= parse_F(tokens, x_val, y_val, index);
                }
                _ => break,
            }
        }
        value
    }

    fn parse_F(tokens: &[&str], x_val: f64, y_val: f64, index: &mut usize) -> f64 {
        let value;
        match tokens.get(*index) {
            Some(&"x") => {
                value = x_val;
                *index += 1;
            }
            Some(&"y") => {
                value = y_val;
                *index += 1;
            }
            Some(&"(") => {
                *index += 1;
                value = parse_E(tokens, x_val, y_val, index);
                if let Some(&")") = tokens.get(*index) {
                    *index += 1;
                }
            }
            Some(token) => {
                value = token.parse::<f64>().unwrap_or(0.0);
                *index += 1;
            }
            None => {
                value = 0.0;
            }
        }
        value
    }

    parse_E(&tokens, x_val, y_val, &mut index)
}


fn evolve_population(population: &[Genome]) -> Vec<Genome> {
    let mut new_population = Vec::with_capacity(POPULATION_SIZE);

    for _ in 0..POPULATION_SIZE {
        let mut child = population[rand::thread_rng().gen_range(0..POPULATION_SIZE)].clone();
        if rand::thread_rng().gen_bool(MUTATION_RATE) {
            mutate(&mut child);
        }
        new_population.push(child);
    }

    new_population
}

fn mutate(genome: &mut Genome) {
    let index = rand::thread_rng().gen_range(0..genome.len());
    genome[index] = rand::thread_rng().gen_range(0..255);
}

fn map_genome_to_phenotype(genome: &Genome) -> String {
    let mut output = String::new();
    let mut symbols = vec!["S"];
    let mut genome_index = 0;
    let mut iteration_count = 0;
    const MAX_ITERATIONS: usize = 1000;

    while let Some(top) = symbols.pop() {
        if iteration_count > MAX_ITERATIONS {
            break;
        }
        if let Some(productions) = GRAMMAR.get(top) {
            let gene = genome[genome_index % genome.len()];
            let production = productions[gene as usize % productions.len()];
            for symbol in production.split_whitespace().rev() {
                symbols.push(symbol);
            }
            genome_index += 1;
        } else {
            output.push_str(top);
            output.push(' ');
        }
        iteration_count += 1;
    }

    output.trim().to_string()
}

fn main() {

    let mut population: Vec<Genome> = (0..POPULATION_SIZE)
        .map(|_| {
            (0..10).map(|_| rand::thread_rng().gen_range(0..255)).collect()
        })
        .collect();

    let mut best_genome: Option<Genome> = None;
    let mut best_fitness: f64 = 0.0;

    println!("pop: {:?}", population);

    let data = vec![
        (0.1, 0.3, 0.31),
        (0.2, 0.6, 0.59),
    ];

    for generation in 0..MAX_GENERATIONS {
        population = evolve_population(&population);

        if let Some(current_best_genome) = population
        .iter()
        .max_by(|a, b| {
            evaluate_fitness(&map_genome_to_phenotype(a), &data)
            .partial_cmp(&evaluate_fitness(&map_genome_to_phenotype(b), &data))
            .unwrap()
        }) {
             let current_best_fitness = evaluate_fitness(&map_genome_to_phenotype(current_best_genome), &data);
             println!("Generation {}: Best Genome (Fitness = {}): {:?}", generation, current_best_fitness, current_best_genome);
             if current_best_fitness > best_fitness {
                best_fitness = current_best_fitness;
                best_genome = Some(current_best_genome.clone());
            }
        }

    }

    if let Some(best) = best_genome {
        println!("Best Individual: {} Genome: {:?}", map_genome_to_phenotype(&best), &best);
        println!("Fitness: {}", best_fitness);
        let new_data_point = (0.3, 0.9);
        let predicted_value = evaluate_expression(&map_genome_to_phenotype(&best), new_data_point.0, new_data_point.1);
        println!("pred: {}", predicted_value);
    } else {
        println!("No best individual found.");
    }
}