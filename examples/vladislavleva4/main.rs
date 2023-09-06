use rand::Rng;
use gers_dev::{grammar, population_mgmt, genome};

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Write, BufRead};

#[derive(Debug, Clone)]
enum Expr {
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Sqrt(Box<Expr>),
    Var(usize),
    Const(f64),
}

fn parse_expression(expr_str: &str) -> Result<Expr, &'static str> {
    let tokens: Vec<&str> = expr_str.split_whitespace().collect();
    let (expr, _) = parse_expr(&tokens, 0)?;
    Ok(expr)
}

fn parse_expr(tokens: &[&str], index: usize) -> Result<(Expr, usize), &'static str> {
    let (mut left, mut i) = parse_term(tokens, index)?;

    while i < tokens.len() {
        match tokens[i] {
            "+" | "-" | "*" | "/" => {
                let op = tokens[i];
                let (right, next_i) = parse_term(tokens, i + 1)?;
                left = match op {
                    "+" => Expr::Add(Box::new(left), Box::new(right)),
                    "-" => Expr::Sub(Box::new(left), Box::new(right)),
                    "*" => Expr::Mul(Box::new(left), Box::new(right)),
                    "/" => Expr::Div(Box::new(left), Box::new(right)),
                    _ => return Err("Unexpected operator"),
                };
                i = next_i;
            }
            _ => break,
        }
    }

    Ok((left, i))
}

fn parse_term(tokens: &[&str], index: usize) -> Result<(Expr, usize), &'static str> {
    if tokens[index].starts_with("x[") {
        let var_index: usize = tokens[index].chars().nth(2).unwrap().to_digit(10).unwrap() as usize;
        return Ok((Expr::Var(var_index), index + 1));
    } else if tokens[index].parse::<f64>().is_ok() {
        return Ok((Expr::Const(tokens[index].parse().unwrap()), index + 1));
    } else if tokens[index] == "pow(" {
        let (base, i) = parse_expr(tokens, index + 1)?;
        if tokens[i] != "," {
            return Err("Expected ',' in pow function");
        }
        let (exponent, next_i) = parse_expr(tokens, i + 1)?;
        if tokens[next_i] != ")" {
            return Err("Expected ')' after pow function");
        }
        return Ok((Expr::Pow(Box::new(base), Box::new(exponent)), next_i + 1));
    } else if tokens[index] == "sqrt(" {
        let (value, i) = parse_expr(tokens, index + 1)?;
        if tokens[i] != ")" {
            return Err("Expected ')' after sqrt function");
        }
        return Ok((Expr::Sqrt(Box::new(value)), i + 1));
    }

    Err("Unexpected token")
}


fn evaluate(expr: &Expr, inputs: &[f64; 5]) -> f64 {
    match expr {
        Expr::Add(a, b) => evaluate(a, inputs) + evaluate(b, inputs),
        Expr::Sub(a, b) => evaluate(a, inputs) - evaluate(b, inputs),
        Expr::Mul(a, b) => evaluate(a, inputs) * evaluate(b, inputs),
        Expr::Div(a, b) => {
            let divisor = evaluate(b, inputs);
            if divisor != 0.0 {
                evaluate(a, inputs) / divisor
            } else {
                std::f64::NAN
            }
        }
        Expr::Pow(a, b) => evaluate(a, inputs).powf(evaluate(b, inputs)),
        Expr::Sqrt(a) => evaluate(a, inputs).sqrt(),
        Expr::Var(index) => inputs[*index],
        Expr::Const(value) => *value,
    }
}

fn vladislavleva4(x: &[f64; 5]) -> f64 {
    10.0 / (5.0 + (x[0]-3.0).powi(2) + (x[1]-3.0).powi(2) + (x[2]-3.0).powi(2) + (x[3]-3.0).powi(2) + (x[4]-3.0).powi(2))
}

fn generate_dataset(samples: usize, range: (f64, f64)) -> Vec<([f64; 5], f64)> {
    let mut rng = rand::thread_rng();
    let mut dataset = Vec::with_capacity(samples);

    for _ in 0..samples {
        let x = [
            rng.gen_range(range.0..range.1),
            rng.gen_range(range.0..range.1),
            rng.gen_range(range.0..range.1),
            rng.gen_range(range.0..range.1),
            rng.gen_range(range.0..range.1),
        ];
        let y = vladislavleva4(&x);
        dataset.push((x, y));
    }

    dataset
}

fn tournament_selection<'a>(population: &'a Vec<genome::Genome>, fitness_values: &Vec<f64>, tournament_size: usize) -> &'a genome::Genome {
    let mut best_individual = &population[rand::random::<usize>() % population.len()];
    let mut best_fitness = f64::MAX; // best_fitness is MAX because we are minimizing fitness

    for _ in 0..tournament_size {
        let index = rand::random::<usize>() % population.len();
        if fitness_values[index] < best_fitness {
            best_fitness = fitness_values[index];
            best_individual = &population[index];
        }
    }

    best_individual
}


fn mutate(individual: &mut Vec<usize>, max_gene_value: usize) {
    let mutation_point = rand::random::<usize>() % individual.len();
    individual[mutation_point] = rand::random::<usize>() % max_gene_value;
}

fn one_point_crossover(parent1: &Vec<usize>, parent2: &Vec<usize>) -> (Vec<usize>, Vec<usize>) {
    let crossover_point = rand::random::<usize>() % parent1.len();
    let mut offspring1 = parent1[..crossover_point].to_vec();
    offspring1.extend(&parent2[crossover_point..]);
    let mut offspring2 = parent2[..crossover_point].to_vec();
    offspring2.extend(&parent1[crossover_point..]);

    (offspring1, offspring2)
}

fn map_genome_to_expression(genome: &genome::Genome, grammar: &grammar::Grammar1) -> String {
    let mut expression = String::new();
    map_rule_to_expression(&mut expression, "Expr", genome, 0, grammar);
    expression
}

fn map_rule_to_expression(expression: &mut String, rule: &str, genome: &[usize], index: usize, grammar: &grammar::Grammar1) -> usize {
    if index >= genome.len() {
        return index;
    }

    if let Some(expansions) = grammar.get(rule) {
        let choice = genome[index] % expansions.len();
        let selected_expansion = &expansions[choice];
        let mut next_index = index + 1;
        for part in selected_expansion.iter() {
            if grammar.contains_key(part) {
                next_index = map_rule_to_expression(expression, part, genome, next_index, grammar);
            } else {
                expression.push_str(&part);
                expression.push(' ');
            }
        }
        next_index
    } else {
        expression.push_str(rule);
        expression.push(' ');
        index + 1
    }
}

fn evaluate_fitness(expression: &str, data: &Vec<([f64; 5], f64)>) -> f64 {
    // println!("The expression: {:?}", expression);
    fn calculate_mse(expression: &str, data: &Vec<([f64; 5], f64)>) -> f64 {
        let mut total_error = 0.0;

        for (x, y) in data {
            // println!("x: {} y: {}", x, y);
            // let predicted_value = evaluate_expression(expression, x);
            match parse_expression(expression) {
                Ok(v) => {
                    let predicted_value = evaluate(&v, x);
                    // let pred = predicted_value.unwrap();
                    let error = predicted_value - y;
                    total_error += error * error;
                }
                Err(_e) => {
                    // return f64::MAX;
                    return 10000.0; // return max fitness for invalid expression
                } 
            };
        }

        let mse = total_error / data.len() as f64;
        mse
    }

    let mse = calculate_mse(expression, data);
    mse
    
}

fn save_dataset_to_file(filename: &str, data: &Vec<([f64; 5], f64)>) {
    let file = File::create(filename).unwrap();
    let mut writer = BufWriter::new(file);

    for &(inputs, output) in data {
        writeln!(writer, "{},{},{},{},{},{}", inputs[0], inputs[1], inputs[2], inputs[3], inputs[4], output).unwrap();
    }
}

fn read_dataset_from_file(filename: &str) -> io::Result<Vec<([f64; 5], f64)>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut dataset = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let values: Vec<f64> = line.split(',').filter_map(|s| s.parse().ok()).collect();
        if values.len() == 6 {
            let inputs = [values[0], values[1], values[2], values[3], values[4]];
            let output = values[5];
            dataset.push((inputs, output));
        }
    }

    Ok(dataset)
}


fn run_algorithm(grammar: &grammar::Grammar1) -> (f64, f64, String) {

    //parameters
    let population_size = 100;
    let max_genome_length = 100;
    let mutation_probability = 0.01;
    let crossover_probability = 0.9;
    let max_generations = 20;
    let tournament_size = 3;
    let max_gene_value = 255;

    let mut population: Vec<genome::Genome> = population_mgmt::random_initialization(population_size, max_genome_length, max_gene_value);
    let mut fitness_values = vec![0.0; population_size];

    let training_data = generate_dataset(1024, (0.05, 6.05));
    let test_data = generate_dataset(5000, (-0.25, 6.35));

    save_dataset_to_file("vlad_train.txt", &training_data);
    save_dataset_to_file("vlad_test.txt", &test_data);

    for generation in 0..max_generations {
        // Evaluate fitness of each individual in the population
        for (i, individual) in population.iter().enumerate() {
            fitness_values[i] = evaluate_fitness(&map_genome_to_expression(individual, &grammar), &training_data);
        }

        let current_best_index = fitness_values.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        let current_best_fitness = fitness_values[current_best_index];
        let current_best_expression = map_genome_to_expression(&population[current_best_index], &grammar);
        println!("Generation {}: Best Genome (Fitness = {}): {:?}", generation, current_best_fitness, current_best_expression);
    
        let mut new_population: Vec<genome::Genome> = Vec::new();
    
        while new_population.len() < population_size {
            let parent1 = tournament_selection(&population, &fitness_values, tournament_size);
            let parent2 = tournament_selection(&population, &fitness_values, tournament_size);
    
            if rand::random::<f64>() < crossover_probability {
                let (child1, child2) = one_point_crossover(parent1, parent2);
                new_population.push(child1);
                new_population.push(child2);
            } else {
                new_population.push(parent1.clone());
                new_population.push(parent2.clone());
            }
        }
    
        for individual in new_population.iter_mut() {
            if rand::random::<f64>() < mutation_probability {
                mutate(individual, max_gene_value);
            }
        }
    
        population = new_population;
    }

    let best_index = fitness_values.iter().enumerate().min_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
    let best_fitness = fitness_values[best_index];
    let best_expression = map_genome_to_expression(&population[best_index], &grammar);

    // Calculate average fitness of the population
    let avg_fitness: f64 = fitness_values.iter().sum::<f64>() / population_size as f64;

    (best_fitness, avg_fitness, best_expression)
}


fn main() {
    let grammar_filename = ".\\grammars\\vlad1.bnf";
    let num_runs = 5; // Number of runs

    let mut best_fitnesses = Vec::new();
    let mut average_fitnesses = Vec::new();
    let mut best_expressions = Vec::new();

    match grammar::read_grammar_from_file(grammar_filename) {
        Ok(grammar) => {
            println!("grammar: {:?}", grammar);
            for _ in 0..num_runs {
                let (best_fitness, avg_fitness, best_expr) = run_algorithm(&grammar);
                best_fitnesses.push(best_fitness);
                average_fitnesses.push(avg_fitness);
                best_expressions.push(best_expr);
            }

            // Analyze results
            let overall_best_fitness = best_fitnesses.iter().cloned().fold(0./0., f64::max);
            let overall_avg_fitness: f64 = average_fitnesses.iter().sum::<f64>() / num_runs as f64;

            println!("Overall Best Fitness: {}", overall_best_fitness);
            println!("Overall Average Fitness: {}", overall_avg_fitness);
            
            let test_data = read_dataset_from_file("vlad_test.txt").unwrap();

            // print best expression from each run
            for (i, expr) in best_expressions.iter().enumerate() {
                println!("Run {}: Best Expression: {}", i+1, expr);
                // println!("Example prediction:");
                // for (x,y) in test_data.iter().take(5) {
                //     match parse_expression(&expr) {
                //         Ok(v) => {
                //             let predicted_value = evaluate(&v, x);
                //             println!("Predicted: {} Actual: {}", predicted_value, y)
                //         }
                //         Err(_e) => {
                //             println!("Invalid expression");
                //         } 
                //     };
                    
                // }
                let test_fitness = evaluate_fitness(&expr, &test_data);
                println!("Run {}: test fitness: {}", i+1, test_fitness);
            }
        },
        Err(e) => {
            println!("Error reading grammar: {}", e);
        }
    }
}
