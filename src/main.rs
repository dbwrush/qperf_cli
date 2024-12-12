#[macro_use]
extern crate lazy_static;
use std::env;
use std::io::{self};
use std::fs;
use std::collections::HashMap;
use std::path::Path;

lazy_static! {
    static ref QUESTION_TYPE_INDICES: HashMap<char, usize> = {
        let mut m = HashMap::new();
        for (i, c) in ['A', 'G', 'I', 'Q', 'R', 'S', 'X', 'V'].iter().enumerate() {
            m.insert(*c, i);
        }
        m
    };
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //get args from console
    let args: Vec<String> = env::args().collect();
    let mut verbose = false;
    //check if the number of args is correct
    if args.len() != 3 {
        if args.len() == 4 {
            if args[3] == "-v" || args[3] == "--verbose"{
                eprintln!("Verbose mode enabled");
                verbose = true;
            } else if args[3] == "-h" || args[3] == "--help" {
                print_help();
                return Ok(());
            } else {
                return Err("Usage: {} path/to/question/sets path/to/quiz/data.csv".into());
            }
        } else {
            return Err("Usage: {} path/to/question/sets path/to/quiz/data.csv".into());
        }
    }

    //get the paths from the args
    let question_sets_dir_path = &args[1];
    let quiz_data_path = &args[2];

    // Validate the paths
    if !Path::new(question_sets_dir_path).exists() {
        return Err(format!("Error: The path to the question sets does not exist: {}", question_sets_dir_path).into());
    }
    if !Path::new(quiz_data_path).exists() {
        return Err(format!("Error: The path to the quiz data does not exist: {}", quiz_data_path).into());
    }

    // Read the directory and sort the entries by name
    let mut entries: Vec<_> = fs::read_dir(question_sets_dir_path)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<_, io::Error>>()?;
    entries.sort();

    let mut question_types_by_round = Vec::new();

    for entry in entries {
        if let Some(ext) = entry.extension() {
            if ext == "rtf" {
                let round = get_round_number(entry.to_str().unwrap()).unwrap();
                let question_types = read_rtf_file(entry.to_str().unwrap())?;
                let round_usize = round as usize;
                while question_types_by_round.len() < round_usize {
                    question_types_by_round.push((question_types_by_round.len(), Vec::new()));
                }
                question_types_by_round.push((round_usize, question_types));
                if verbose {
                    eprintln!("Found RTF file: {:?}", entry);
                }
            }
        }
    }
    if verbose {
        eprintln!("{:?}", question_types_by_round);
    }
    // Sort the question types by round number
    question_types_by_round.sort_by_key(|k| k.0);

    // Now you have a sorted 2D array of question types for each round
    let question_types: Vec<Vec<char>> = question_types_by_round.into_iter().map(|(_, qt)| qt).collect();
    let mut mut_question_types: Vec<Vec<char>> = Vec::new();
    let mut mut_inner_vec: Vec<char> = Vec::new();
    let inner_vec = question_types.iter().next().unwrap();
    for (i, t) in inner_vec.iter().enumerate() {
        if i % 20 == 0 && i > 0 {
            mut_question_types.push(mut_inner_vec.clone());
            mut_inner_vec.clear();
        }
        mut_inner_vec.push(t.clone());
    }
    let question_types = mut_question_types;

    if verbose {
        eprintln!("{:?}", question_types);
    }
    let mut quiz_records = vec![];
    //read quiz data file
    match read_csv_file(quiz_data_path) {
        Ok(records) => {
            quiz_records = records.clone();
            /*println!("CSV Content:");
            for record in records {
                println!("{:?}", record);
            }*/
        }
        Err(e) => eprintln!("Error reading CSV file: {}", e),
    }

    let records = filter_records(quiz_records);
    let quizzer_names = get_quizzer_names(records.clone());
    if verbose {
        eprintln!("Quizzer Names: {:?}", quizzer_names);
    }
    let num_quizzers = quizzer_names.len();
    let num_question_types = QUESTION_TYPE_INDICES.len();

    let mut attempts: Vec<Vec<f32>> = vec![vec![0.0; num_question_types]; num_quizzers];
    let mut correct_answers: Vec<Vec<f32>> = vec![vec![0.0; num_question_types]; num_quizzers];
    let mut bonus_attempts: Vec<Vec<f32>> = vec![vec![0.0; num_question_types]; num_quizzers];
    let mut bonus: Vec<Vec<f32>> = vec![vec![0.0; num_question_types]; num_quizzers];

    update_arrays(records, &quizzer_names, question_types, &mut attempts, &mut correct_answers, &mut bonus_attempts, &mut bonus, verbose);

    print_results(quizzer_names, attempts, correct_answers, bonus_attempts, bonus);

    Ok(())
}

fn print_results(quizzer_names: Vec<String>, attempts: Vec<Vec<f32>>, correct_answers: Vec<Vec<f32>>, bonus_attempts: Vec<Vec<f32>>, bonus: Vec<Vec<f32>>) {
    // Print the header
    print!("Quizzer\t");
    let mut question_types_list: Vec<_> = QUESTION_TYPE_INDICES.keys().collect();
    question_types_list.sort();
    for question_type in &question_types_list {
        print!("{} QA\t{} QC\t{} BA\t{} BC\t", question_type, question_type, question_type, question_type);
    }
    println!();

    // Print the results for each quizzer
    for (i, quizzer_name) in quizzer_names.iter().enumerate() {
        print!("{}\t", quizzer_name);
        for question_type in &question_types_list {
            let question_type_index = *QUESTION_TYPE_INDICES.get(question_type).unwrap_or(&0);
            print!("{:.1}\t{:.1}\t{:.1}\t{:.1}\t",
                   attempts[i][question_type_index],
                   correct_answers[i][question_type_index],
                   bonus_attempts[i][question_type_index],
                   bonus[i][question_type_index]);
        }
        println!();
    }
}

fn update_arrays(records: Vec<csv::StringRecord>, quizzer_names: &Vec<String>, question_types: Vec<Vec<char>>, attempts: &mut Vec<Vec<f32>>, correct_answers: &mut Vec<Vec<f32>>, bonus_attempts: &mut Vec<Vec<f32>>, bonus: &mut Vec<Vec<f32>>, verbose: bool) {
    let mut warns = false;
    for record in records {
        if verbose {
            eprintln!("{:?}", record);
        }
        // Split the record by commas to get the columns
        let columns: Vec<&str> = record.into_iter().collect();
        // Get the event type code, quizzer name, and question number
        let event_code = columns.get(10).unwrap_or(&"");
        if verbose {
            eprint!("ECode: {} ", event_code);
        }
        let quizzer_name = columns.get(7).unwrap_or(&"");
        if verbose {
            eprint!("QName: {} ", quizzer_name);
        }
        let round_number = columns.get(4).unwrap_or(&"").trim_matches('\'').parse::<usize>().unwrap_or(0) - 1;
        if verbose {
            eprint!("RNum: {} ", round_number + 1);
        }
        let question_number = columns.get(5).unwrap_or(&"").trim_matches('\'').parse::<usize>().unwrap_or(0) - 1;
        if verbose {
            eprint!("QNum: {} ", question_number + 1);
        }
        // Check if the round_number and question_number are within the bounds of the question_types array
        if round_number >= question_types.len() || question_number >= question_types[round_number].len() {
            warns = true;
            if verbose {
                eprintln!("Warning: No question type found for round {}, question {}. Skipping record.", round_number + 1, question_number + 1);
            }
            continue;
        }
        // Find the index of the quizzer in the quizzer_names array
        let quizzer_index = quizzer_names.iter().position(|n| n == quizzer_name).unwrap_or(0);
        // Get the question type based on question number
        let mut question_type = 'G';
        if (question_number + 1) != 21 {
            question_type = question_types[round_number as usize][question_number];
        }
        let question_type = question_type;
        if verbose {
            eprint!("QType: {} ", question_type);
        }
        // Find the index of the question type in the arrays
        let question_type_index = *QUESTION_TYPE_INDICES.get(&question_type).unwrap_or(&0);
        if verbose {
            eprintln!("QTInd: {} ", question_type_index);
        }
        // Update the arrays based on the event type code
        match *event_code {
            "'TC'" => {
                attempts[quizzer_index][question_type_index] += 1.0;
                correct_answers[quizzer_index][question_type_index] += 1.0;
            }
            "'TE'" => {
                attempts[quizzer_index][question_type_index] += 1.0;
            }
            "'BC'" => {
                bonus_attempts[quizzer_index][question_type_index] += 1.0;
                bonus[quizzer_index][question_type_index] += 1.0;
            }
            "'BE'" => {
                bonus_attempts[quizzer_index][question_type_index] += 1.0;
            }
            _ => {}
        }
    }
    if warns {
        eprint!("Warning: Some records were skipped due to missing question sets");
    }
}

fn get_quizzer_names(records: Vec<csv::StringRecord>) -> Vec<String> {
    let mut quizzer_names_by_team: Vec<Vec<String>> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for record in records {
        // Split the record by commas to get the columns
        let columns: Vec<&str> = record.into_iter().collect();
        // Get the quizzer name and team number
        let quizzer_name = columns.get(7).unwrap_or(&"").to_string();
        let team_number = columns.get(8).unwrap_or(&"").parse::<usize>().unwrap_or(0);
        // Ensure the team number is within the range of the 2D array
        if team_number >= quizzer_names_by_team.len() {
            quizzer_names_by_team.resize(team_number + 1, Vec::new());
        }
        // Add the quizzer name to the corresponding team if it hasn't been seen before
        if seen_names.insert(quizzer_name.clone()) {
            quizzer_names_by_team[team_number].push(quizzer_name);
        }
    }
    // Flatten the 2D array into a single array
    let quizzer_names: Vec<String> = quizzer_names_by_team.into_iter().flatten().collect();

    quizzer_names
}

fn filter_records(records: Vec<csv::StringRecord>) -> Vec<csv::StringRecord> {
    let mut filtered_records = Vec::new();
    let event_codes = vec!["'TC'", "'TE'", "'BC'", "'BE'"]; // event type codes

    for record in records {
        // Split the record by commas to get the columns
        let columns: Vec<&str> = record.into_iter().collect();
        // Check if the 5th column matches the round number and 11th column contains the event type codes
        if columns.get(10).map_or(false, |v| event_codes.contains(&v)) {
            filtered_records.push(csv::StringRecord::from(columns));
        }
    }

    // Print the filtered records for debugging
    /*println!("Filtered Records:");
    for record in &filtered_records {
        println!("{:?}", record);
    }*/

    filtered_records
}

fn get_round_number(path: &str) -> io::Result<i8> {
    let content = fs::read_to_string(path)?;
    let re = regex::Regex::new(r"SET #(\d+)").unwrap();
    match re.captures(&content) {
        Some(caps) => {
            match caps.get(1).unwrap().as_str().parse::<i8>() {
                Ok(num) => Ok(num - 1),
                Err(_) => {
                    eprintln!("Error: Invalid round number: {}", caps.get(1).unwrap().as_str());
                    std::process::exit(1);
                }
            }
        },
        None => {
            eprintln!("Error: No round number found in file");
            std::process::exit(1);
        }
    }
}

fn read_rtf_file(path: &str) -> io::Result<Vec<char>> {
    let content = fs::read_to_string(path)?;
    //println!("RTF Content:\n{}", content);
    let mut question_types = Vec::new();
    let parts: Vec<_> = content.split("\\tab").collect();
    for (i, part) in parts.iter().enumerate() {
        if i % 2 == 0 && !part.is_empty() {
            let chars: Vec<char> = part.chars().collect();
            let len = chars.len();
            if len > 1 {
                question_types.push(chars[len - 2]);
            }
        }
    }

    Ok(question_types)
}

fn read_csv_file(path: &str) -> Result<Vec<csv::StringRecord>, csv::Error> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut records = Vec::new();

    for result in reader.records() {
        let record = result?;
        records.push(record);
    }

    Ok(records)
}

fn print_help() {
    println!("qperformance - A tool for analyzing quiz performance data");
    println!();
    println!("USAGE:");
    println!("    qperformance path/to/question/sets path/to/quiz/data.csv");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Prints help information");
    println!("    -v, --verbose    Enables verbose mode");
    println!();
    println!("ARGS:");
    println!("    <question_sets>  The path to the directory containing the question sets");
    println!("    <quiz_data>      The path to the CSV file containing the quiz data");
}