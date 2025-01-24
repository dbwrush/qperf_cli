# qperf_cli

**qperf_cli** is a command-line tool for analyzing Bible Quizzers' performance on all question types across multiple rounds. It processes `.csv` logs generated by QuizMachine alongside `.rtf` question sets, providing a detailed performance analysis of individual quizzers.

**Looking for a GUI version?** Check out [qperformance](https://github.com/dbwrush/qperformance) for all the same functionality in a graphical interface.

---

## Features

- Analyze quizzer performance by question type.
- Tracks how often each quizzer:
  - Attempts each type of question.
  - Answers each type correctly.
  - Attempts and answers bonus questions.
- Choose which question types to include in the report
- Outputs results directly to the terminal or to a file in a CSV format, compatible with tools like Excel.
- Customizable delimiters between values. Default is `,`.

## Flags

- Verbose `-v` or `--verbose`
  - Display logs and debug messages as the program runs
- Types `-t` or `--types`
  - Select which question types to be included in output. Default to ALL
  - Types are A, G, I, Q, R, S, X, V, and M.
  - M is a total of types Q, R, and V
- Delim `-d` or `--delim`
  - Specify delimiter for output file. Default to `,`
- Help `-h` or `--help`
  - Display help/usage menu

---

## Getting Started

### Download Pre-Compiled Builds

Compiled binaries for **qperf_cli** are available for:

- **Windows PCs** (x86, Intel/AMD CPUs)
- **Linux PCs** (x86, Intel/AMD CPUs)

Visit the [Releases](https://github.com/dbwrush/qperf_cli/releases) page to download the latest version.

> **Note:**  
> Most Windows PCs use Intel/AMD CPUs and should work with the provided build. For macOS or ARM-based devices, you’ll need to build the program from source (see below).

---

### Usage Instructions

1. **Prepare Input Files**:
   - A `.csv` file generated by QuizMachine containing round logs.
   - One or more `.rtf` question set files, where question numbers correspond to round numbers in the log.

2. **Run the Program**:
   ```bash
   ./qperf.exe [OPTIONS] <question_sets> <quiz_data>
   ```
   Replace `<question_sets>` with the path to your `.rtf` file(s) or directory, and `<quiz_data>` with the path to your `.csv` log file.

3. **Optional Flags**:
   - `-v` or `--verbose`: Display detailed processing information.
   - `-t` or `--types`: Specify question types to analyze (e.g., `-t ag` for According To and General only).
   
   Available types:
   (A): According To
   (G): General
   (I): In What Book and Chapter
   (Q): Quote
   (R): Reference
   (S): Situation
   (X): Context
   (V): Verse
   (M): Total of Quote, Reference, and Verse

4. **Output the Results to a CSV File** (optional):
   ```bash
   ./qperf.exe <question_sets> <quiz_data> > output.csv
   ```

### Examples

Basic usage:
```bash
./qperf.exe quiz_sets/ event_logs.csv
```

Customizing analysis:
```bash
./qperf.exe -v -t ag quiz_sets/ event_logs.csv > report.csv
```

This creates a file `report.csv` with detailed quizzer statistics for According To and General question types
---

## Help Information

To view the help menu, run:
```bash
./qperf.exe --help
```

This will display available flags, options, and usage examples.

---

## Building From Source

If the pre-compiled binaries do not meet your needs, or you’re on a platform like macOS or an ARM-based device, you can build the program from source.

### Prerequisites

- Install [Rust](https://www.rust-lang.org/tools/install) on your system.

### Build Instructions

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/dbwrush/qperf_cli.git
   cd qperf_cli
   ```

2. **Build the Project**:
   ```bash
   cargo build --release
   ```

3. **Run the Program**:
   The executable will be located in the `target/release` directory:
   ```bash
   ./target/release/qperf.exe [OPTIONS] <question_sets> <quiz_data>
   ```

---

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for improvements or bug fixes.
