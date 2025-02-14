extern crate qperf_lib;

use std::env;
use qperf_lib::{qperf, get_question_types};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //get args from console
    let args: Vec<String> = env::args().collect();
    let mut verbose = false;
    let mut types: Vec<char> = Vec::new();
    let mut tourn: String = "".to_string();

    #[derive(PartialEq)]
    enum ExpectType {
        None,
        Types,
        Delim,
        Tourn
    }

    //parse args
    //first two arguments should be the path to the question sets and the path to the quiz data
    //if the first two arguments are invalid, print help info.
    //optional flags (there may be more than one, passed in any order):
    //  -v or --verbose: enables verbose mode
    //  -t or --types: specifies the types of questions to analyze (question types must follow the flag as a string of chars)
    //  -h or --help: prints help information (check for this first, as it overrides all other flags including file paths)
    //  -d or --delim: specifies the delimiter for the CSV file (default is ',')
    //  -n or --name: specifies the name of the tournament to filter data by
    
    let mut question_sets_path = None;
    let mut quiz_data_path = None;
    let mut expect_type = ExpectType::None;
    let mut delim = ",";
    let mut display_rounds = false;
    for i in 1..args.len() {
        match args[i].as_str() {
            _ if expect_type == ExpectType::Types => {
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
                expect_type = ExpectType::None;
            },
            _ if expect_type == ExpectType::Delim => {
                delim = &args[i];
                expect_type = ExpectType::None;

                if delim == "\\t" { //Make sure tab characters are properly interpreted
                    delim = "\t";
                }

                //Perform error checking on the delimiter argument
                if delim.len() > 5 || delim.contains("/") || delim.contains("\\") {
                    eprintln!("Error: Invalid delimiter '{}'.", delim);
                    eprintln!("Delimiters should be short and not contain file path characters.");
                    return Ok(());
                }
            },
            _ if expect_type == ExpectType::Tourn => {
                if args[i].contains("/") || args[i].contains("\\") {
                    eprintln!("Error: Unexpected argument '{}'.", args[i]);
                    eprintln!("Did you forget to specify the tournament name after -n or --name?");
                    print_help();
                    return Ok(());
                }
                tourn = args[i].clone();
                expect_type = ExpectType::None;
            },
            "-v" | "--verbose" => verbose = true,
            "-t" | "--types" => {
                if i + 1 < args.len() {
                    expect_type = ExpectType::Types;                  
                } else {
                    eprintln!("Error: Missing question types after -t or --types.");
                    return Ok(());
                }
            },
            "-h" | "--help" => {
                print_help();
                return Ok(());
            },
            "-d" | "--delim" => {
                if i + 1 < args.len() {
                    expect_type = ExpectType::Delim;
                } else {
                    eprintln!("Error: Missing delimiter after -d or --delim.");
                    return Ok(());
                }
            }
            "-n" | "--name" => {
                if i + 1 < args.len() {
                    expect_type = ExpectType::Tourn;
                } else {
                    eprintln!("Error: Missing name after -n or --name.");
                    return Ok(());
                }
            }
            "-r" | "--round" => display_rounds = true,
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
    match qperf(&question_sets_path.unwrap(), &quiz_data_path.unwrap(), verbose, types, delim.to_string(), tourn, display_rounds) {
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
    println!("    -d, --delim      Specifies the delimiter for the CSV file (default is ',').");
    println!("    -n, --name       Filters to only include data from the specified tournament.");
    println!("    -r, --round      Displays the round number in the output.");
    println!();
    println!("NOTES:");
    println!("    - The <question_sets> and <quiz_data> arguments are required and must appear.");
    println!("    - Flags (-v, -t, etc) can be specified in any order relative to the positional arguments.");
    println!();
    println!("EXAMPLES:");
    println!("    qperformance /path/to/questions /path/to/quiz.csv -v");
    println!("    qperformance -t ag /path/to/questions /path/to/quiz.csv");
    println!("    qperformance --help");
}
