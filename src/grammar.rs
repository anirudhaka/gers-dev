use std::collections::HashMap;
use std::fs::{File, read_to_string};
use std::io::{self, BufRead};
use std::path::Path;
use lazy_static::lazy_static;

pub type Grammar = HashMap<String, Vec<String>>;
pub type Genome = Vec<i32>;

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

pub fn parse_grammar(filename: &str) -> io::Result<Grammar> {
    let path = Path::new(filename);
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut grammar = Grammar::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split("::=").collect();
        if parts.len() == 2 {
            let non_terminal = parts[0].trim().to_string();
            let productions: Vec<String> = parts[1].split('|').map(|s| s.trim().to_string()).collect();
            grammar.insert(non_terminal, productions);
        }
    }

    Ok(grammar)
}

pub type Grammar1 = HashMap<String, Vec<Vec<String>>>;

pub fn read_grammar_from_file(filename: &str) -> Result<Grammar1, std::io::Error> {
    let content = read_to_string(filename)?;

    let mut grammar: HashMap<String, Vec<Vec<String>>> = HashMap::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split("::=").collect();
        if parts.len() != 2 {
            continue;
        }

        let rule_name = parts[0].trim().to_string();
        let expansions: Vec<Vec<String>> = parts[1]
            .split('|')
            .map(|expansion| expansion.trim().split_whitespace().map(String::from).collect())
            .collect();

        grammar.insert(rule_name, expansions);
    }

    Ok(grammar)
}

pub fn is_recursive(non_terminal: &str, production: &str, grammar: &Grammar) -> bool {
    if production.contains(non_terminal) {
        return true;
    }
    for symbol in grammar.keys() {
        if production.contains(symbol) {
            for prod in &grammar[symbol] {
                if is_recursive(non_terminal, prod, grammar) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn calculate_arity(production: &str, grammar: &Grammar) -> usize {
    grammar.keys().filter(|&non_terminal| production.contains(non_terminal)).count()
}

pub fn map_genome_to_phenotype(genome: &Genome) -> String {
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

pub fn genome_to_phenotype(genome: &Vec<i32>, grammar: &Grammar) -> String {
    let mut phenotype = "S".to_string();
    let mut gene_index = 0;
    
    while gene_index < genome.len() {
        if let Some(productions) = grammar.get(&phenotype) {
            let rule_index = genome[gene_index] as usize % productions.len();
            phenotype = productions[rule_index].clone();
            gene_index += 1;
        } else {
            break;
        }
    }
    
    phenotype
}