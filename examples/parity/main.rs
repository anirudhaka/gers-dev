use rand::Rng;
use lazy_static::lazy_static;
use std::collections::HashMap;


// Genome Representation
type Genome = Vec<u8>;

// Grammar Representation
// For simplicity the grammar is saved as a hashmap and not read from a grammar file.
lazy_static! {
    static ref GRAMMAR: HashMap<&'static str, Vec<&'static str>> = {
        let mut map = HashMap::new();
        map.insert("S", vec!["E"]);
        map.insert("E", vec!["E OR T", "T"]);
        map.insert("T", vec!["T AND F", "F"]);
        map.insert("F", vec!["NOT F", "A", "B", "C"]);
        map
    };
}


// Genome-to-Phenotype Mapping
fn map_genome_to_phenotype(genome: &Genome) -> String {
    let mut output = String::new();
    let mut symbols = vec!["S"];
    let mut genome_index = 0;

    while let Some(top) = symbols.pop() {
        if let Some(productions) = GRAMMAR.get(top) {
            let gene = genome[genome_index % genome.len()];  // Cyclically use the genome
            let production = productions[gene as usize % productions.len()];
            for symbol in production.split_whitespace().rev() {
                symbols.push(symbol);
            }
            genome_index += 1;  // Move to the next gene in the genome
        } else {
            output.push_str(top);
            output.push(' ');
        }
    }

    output.trim().to_string()
}


// Fitness Evaluation
fn evaluate_fitness(expression: &str) -> i32 {
    let combinations = [
        (false, false, false),
        (false, false, true),
        (false, true, false),
        (false, true, true),
        (true, false, false),
        (true, false, true),
        (true, true, false),
        (true, true, true),
    ];

    let mut correct_count = 0;

    for (a, b, c) in &combinations {
        let result = evaluate_expression(expression, *a, *b, *c);
        if result == (*a ^ *b ^ *c) {
            correct_count += 1;
        }
    }

    correct_count
}

// evaluate the Boolean expression
fn evaluate_expression(expression: &str, a: bool, b: bool, c: bool) -> bool {
    // Tokenization
    let tokens: Vec<&str> = expression.split_whitespace().collect();

    // Postfix Conversion
    let postfix = infix_to_postfix(&tokens);

    // Evaluation
    match evaluate_postfix(&postfix, a, b, c) {
        Some(result) => result,  // Handle the Some(bool) case
        None => {
            // Handle the None case
            // eprintln!("Error: Malformed postfix expression or evaluation error.");
            false
        }
    }
}

fn infix_to_postfix<'a>(tokens: &'a [&'a str]) -> Vec<&'a str> {
    let mut output = Vec::new();
    let mut stack = Vec::new();

    for &token in tokens {
        match token {
            "A" | "B" | "C" => output.push(token),
            "AND" | "OR" => {
                while let Some(&top) = stack.last() {
                    if top == "AND" || top == "OR" || top == "NOT" {
                        output.push(stack.pop().unwrap());
                    } else {
                        break;
                    }
                }
                stack.push(token);
            }
            "NOT" => stack.push(token),
            "(" => stack.push(token),
            ")" => {
                while let Some(&top) = stack.last() {
                    if top != "(" {
                        output.push(stack.pop().unwrap());
                    } else {
                        stack.pop();
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    while let Some(op) = stack.pop() {
        output.push(op);
    }

    output
}


fn evaluate_postfix(postfix: &[&str], a: bool, b: bool, c: bool) -> Option<bool> {
    let mut stack = Vec::new();

    for &token in postfix {
        match token {
            "A" => stack.push(a),
            "B" => stack.push(b),
            "C" => stack.push(c),
            "AND" => {
                if stack.len() < 2 {
                    return None;
                }
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                stack.push(left && right);
            }
            "OR" => {
                if stack.len() < 2 {
                    return None;
                }
                let right = stack.pop().unwrap();
                let left = stack.pop().unwrap();
                stack.push(left || right);
            }
            "NOT" => {
                if stack.is_empty() {
                    return None;
                }
                let operand = stack.pop().unwrap();
                stack.push(!operand);
            }
            _ => {}
        }
    }

    if stack.len() == 1 {
        Some(stack.pop().unwrap())
    } else {
        None
    }
}


// Evolutionary Operations
fn mutate(genome: &mut Genome) {
    let index = rand::thread_rng().gen_range(0..genome.len());
    genome[index] = rand::thread_rng().gen_range(0..255);
}

// Population Management
const POPULATION_SIZE: usize = 10;
const MUTATION_RATE: f64 = 0.01;

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

// Termination Criteria
const MAX_GENERATIONS: usize = 10;

fn main() {
    // Initialize population
    let mut population: Vec<Genome> = (0..POPULATION_SIZE)
        .map(|_| {
            (0..10).map(|_| rand::thread_rng().gen_range(0..255)).collect()
        })
        .collect();

    let mut best_genome: Option<Genome> = None;
    let mut best_fitness = 0;

    for generation in 0..MAX_GENERATIONS {
        population = evolve_population(&population);

        // Find the best genome of this generation
        if let Some(current_best_genome) = population.iter().max_by_key(|genome| evaluate_fitness(&map_genome_to_phenotype(genome))) {
            let current_best_fitness = evaluate_fitness(&map_genome_to_phenotype(current_best_genome));
            println!("Generation {}: Best Genome (Fitness = {}): {:?}", generation, evaluate_fitness(&map_genome_to_phenotype(current_best_genome)), current_best_genome);
            if current_best_fitness > best_fitness {
                best_fitness = current_best_fitness;
                best_genome = Some(current_best_genome.clone());
            }
        }
    }

    // Print the best individual at the end of the run
    if let Some(best) = best_genome {
        println!("Best Individual: {} Genome: {:?}", map_genome_to_phenotype(&best), &best);
        println!("Fitness: {}", best_fitness);
    } else {
        println!("No best individual found.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infix_to_postfix() {
        let tokens = vec!["A", "AND", "B", "OR", "C"];
        let postfix = infix_to_postfix(&tokens);
        assert_eq!(postfix, vec!["A", "B", "AND", "C", "OR"]);
    }

    #[test]
    fn test_evaluate_postfix() {
        let postfix = vec!["A", "B", "AND", "C", "OR"];
        let result = evaluate_postfix(&postfix, true, false, true);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_evaluate_expression() {
        let expression = "A AND B OR C";
        let result = evaluate_expression(expression, true, false, true);
        assert_eq!(result, true);
    }

    #[test]
    fn test_map_genome_to_phenotype() {
        let genome = vec![0, 1, 2, 3, 4];
        let phenotype = map_genome_to_phenotype(&genome);
        println!("{}", phenotype);
        assert_eq!(phenotype, "NOT NOT A AND B");
    }

    // #[test]
    // fn test_evaluate_fitness() {
    //     let expression = "A AND B OR C";
    //     let fitness = evaluate_fitness(&expression);
    //     assert_eq!(fitness, 4);
    // }
}
