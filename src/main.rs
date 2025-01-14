extern crate lazy_static;
use std::env;

use qperf::{get_question_types, qperf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //get args from console
    let args: Vec<String> = env::args().collect();
    let mut verbose = false;
    let mut types: Vec<char> = Vec::new();

    //parse args
    //first two arguments should be the path to the question sets and the path to the quiz data
    //if the first two arguments are invalid, print help info.
    //optional flags (there may be more than one, passed in any order):
    //  -v or --verbose: enables verbose mode
    //  -t or --types: specifies the types of questions to analyze (question types must follow the flag as a string of chars)
    //  -h or --help: prints help information (check for this first, as it overrides all other flags including file paths)
    
    let mut question_sets_path = None;
    let mut quiz_data_path = None;
    let mut types_expected = false;
    for i in 1..args.len() {
        match args[i].as_str() {
            "-v" | "--verbose" => verbose = true,
            "-t" | "--types" => {
                if i + 1 < args.len() {
                    types_expected = true;                  
                } else {
                    eprintln!("Error: Missing question types after -t or --types.");
                    return Ok(());
                }
            },
            "-h" | "--help" => {
                print_help();
                return Ok(());
            },
            _ if types_expected => {
                //check if argument is actually a file path, if so give an error message.
                if args[i].contains("/") || args[i].contains("\\") {
                    eprintln!("Error: Unexpected argument '{}'.", args[i]);
                    eprintln!("Did you forget to specify the question types after -t or --types?");
                    print_help();
                    return Ok(());
                }
                for c in args[i].chars() {
                    //check that c (or its uppercase equivalent) is a valid question type found in get_question_types()
                    if !get_question_types().contains(&c.to_ascii_uppercase()) {
                        eprintln!("Error: Invalid question type '{}'.", c);
                        eprintln!("Valid question types are: {:?}", get_question_types());
                        return Ok(());
                    }
                    types.push(c.to_ascii_uppercase());
                }
                types_expected = false;
            },
            _ => {
                if question_sets_path.is_none() {
                    question_sets_path = Some(args[i].clone());
                } else if quiz_data_path.is_none() {
                    quiz_data_path = Some(args[i].clone());
                } else {
                    eprintln!("Error: Unexpected argument '{}'.", args[i]);
                    print_help();
                    return Ok(());
                }
            }
        }
    }
    
    if question_sets_path.is_none() || quiz_data_path.is_none() {
        eprintln!("Error: Missing paths to question sets or quiz data.");
        print_help();
        return Ok(());
    }

    if types.is_empty() {
        types = get_question_types();
    }

    if verbose {
        eprintln!("Using types: {:?}", types);
    }

    //run qperf function
    match qperf(&question_sets_path.unwrap(), &quiz_data_path.unwrap(), verbose, types) {
        Ok(result) => {
            //print result to standard output. Result will contain newline characters.
            //The Vec<String> is warnings, and the String is the result of the analysis
            for warning in result.0 {
                eprintln!("{}", warning);
            }
            println!("{}", result.1);
        },
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(e);
        }
    }
    Ok(())
}

fn print_help() {
    println!("qperformance - A tool for analyzing quiz performance data");
    println!();
    println!("USAGE:");
    println!("    qperformance [OPTIONS] <question_sets> <quiz_data>");
    println!();
    println!("ARGS:");
    println!("    <question_sets>  The path to the directory containing the question sets.");
    println!("    <quiz_data>      The path to the CSV file containing the quiz data.");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Prints help information and exits.");
    println!("    -v, --verbose    Enables verbose mode for detailed output.");
    println!("    -t, --types      Specifies the question types to analyze (e.g., '-t ab').");
    println!();
    println!("NOTES:");
    println!("    - The <question_sets> and <quiz_data> arguments are required and must appear.");
    println!("    - Flags (-v, -t) can be specified in any order relative to the positional arguments.");
    println!();
    println!("EXAMPLES:");
    println!("    qperformance /path/to/questions /path/to/quiz.csv -v");
    println!("    qperformance -t ag /path/to/questions /path/to/quiz.csv");
    println!("    qperformance --help");
}
