# MiniLang Compiler

[![Build Status](https://img.shields.io/badge/build-passing-success)](https://github.com/heetabhanushali/minilang_compiler)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/wasm-ready-purple)](https://webassembly.org/)

A **statically-typed, compiled programming language** built entirely in Rust, featuring a unique syntax with Pascal-style logical operators (`AND`/`OR`/`NOT`) and a complete compiler pipeline from source to native executable.

## Web Playground

The online playground compiles MiniLang to C using WebAssembly, then executes the generated C code via the [Piston API](https://emkc.org/api/v2/piston/runtimes) — a free, open-source code execution engine.

**Note:** Code execution requires an internet connection. The Piston API runs your code in a sandboxed environment with:
- 5 second execution timeout
- 10 second compilation timeout
- Memory limits for safety

**[Try MiniLang in your browser →](https://minilang-playground.vercel.app/)**

Experience the interactive playground with real-time compilation, AST visualization, static analysis, and optimization statistics.

## Features

### Language Features
- **Unique Readable Syntax**: `AND`/`OR`/`NOT` operators for better readability
- **Static Type System**: Type safety with `int`, `float`, `string`, `bool`, and arrays
- **Modern Control Flow**: `if`/`else`, `while`, `do-while`, `for` loops, `break`/`continue`
- **Functions**: First-class functions with return types and recursion support
- **String Interpolation**: Embed expressions directly in strings with `{}`

### Compiler Features
- **Complete Pipeline**: Lexer → Parser → Type Checker → Optimizer → Code Generator
- **4 Optimization Techniques**:
  - Constant Folding (evaluate compile-time expressions)
  - Dead Code Elimination (remove unreachable code)
  - Constant Propagation (replace variables with known values)
  - Strength Reduction (replace expensive ops with cheaper ones)
- **Beautiful Error Messages**: Context-aware errors with suggestions using `miette`
- **Multiple Backends**: Compile to C or run directly via WebAssembly
- **Interactive Debugging**: Step through compilation phases

### Static Analysis
- **Complexity Metrics**: Cyclomatic, Cognitive, Halstead, Nesting Depth, Fan-out
- **Quality Ratings**: A/B/C/D/F grades per function based on complexity thresholds
- **Actionable Warnings**: Suggestions to improve code maintainability
- **JSON Output**: Machine-readable reports for CI/CD integration

## Quick Start

### Web Playground

Visit the [MiniLang Playground](https://minilang-playground.vercel.app/) to start coding immediately, or run locally:

```bash
cd playground
python3 -m http.server 8000
# Visit http://localhost:8000
```

### Command Line

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/heetabhanushali/minilang_compiler.git
cd minilang_compiler
cargo build --release

# Install globally
cargo install --path .

# Run your first program
echo 'func main() { display "Hello, MiniLang!"; }' > hello.mini
minilang run hello.mini
```

## Language Syntax

### Hello World
```
func main() {
    display "Hello, World!";
}
```

### Variables and Types
```
func main() {
    let x: int = 42;
    let pi: float = 3.14159;
    let name: string = "MiniLang";
    let is_awesome: bool = true;
    let numbers: int[5] = [1, 2, 3, 4, 5];
    
    # String interpolation
    display "The answer is {x}";
}
```

### Control Flow
```
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}

func main() {
    # Loops with logical operators
    for i = 1; i <= 5; i = i + 1 {
        if i % 2 == 0 AND i != 4 {
            display "Even: ", i;
        }
    }
    
    # display outputs to console, send returns values
    let result: int = factorial(5);
    display "5! = ", result;
}
```

## CLI Usage

### Commands

| Command | Description |
|---------|-------------|
| `minilang compile <file>` | Compile to executable |
| `minilang run <file>` | Compile and run immediately |
| `minilang check <file>` | Type-check without compiling |
| `minilang analyze <file>` | Run static analysis |
| `minilang ast <file>` | Display Abstract Syntax Tree |
| `minilang tokens <file>` | Display token stream |
| `minilang stats <file>` | Show compilation statistics |
| `minilang clean` | Remove generated files |

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <name>` | Output executable name |
| `-O, --opt <level>` | Optimization level (0-2) |
| `-d, --detail` | Show compilation steps |
| `--keep-c` | Keep intermediate C file |
| `--json` | JSON output (for analyze) |

### Examples

```bash
# Compile with optimizations
minilang compile program.mini -O 2

# Check for errors without compiling
minilang check program.mini

# Run static analysis
minilang analyze program.mini

# JSON output for CI/CD
minilang analyze program.mini --json

# View the AST
minilang ast program.mini

# See compilation statistics with timing
minilang stats program.mini --time
```

## Static Analysis

MiniLang includes a built-in static analyzer that calculates complexity metrics for every function.

### Metrics

| Metric | What It Measures |
|--------|-----------------|
| **Cyclomatic Complexity** | Independent paths through code |
| **Cognitive Complexity** | Human-perceived difficulty (SonarSource-style) |
| **Halstead Metrics** | Volume, Difficulty, Effort based on operators/operands |
| **Nesting Depth** | Maximum depth of nested blocks |
| **Fan-out** | Number of distinct functions called |
| **Lines of Code** | Non-empty, non-comment lines per function |

### Rating System

Each function receives a grade based on cyclomatic and cognitive complexity:

| Rating | Cyclomatic | Cognitive | Meaning |
|--------|-----------|-----------|---------|
| **A** | 1 – 5 | 0 – 5 | Excellent — simple and maintainable |
| **B** | 6 – 10 | 6 – 10 | Good — acceptable complexity |
| **C** | 11 – 20 | 11 – 15 | Moderate — consider refactoring |
| **D** | 21 – 50 | 16 – 30 | Complex — should be refactored |
| **F** | 51+ | 31+ | Very complex — must be refactored |

### Warnings

The analyzer flags potential issues:
- Cyclomatic complexity > 10
- Cognitive complexity > 15
- Nesting depth > 3
- Parameters > 5
- Fan-out > 8
- LOC > 50


## Architecture

```
Source (.mini) → Lexer → Parser → Type Checker → Optimizer → Code Gen → C Code → GCC → Executable
                   ↓        ↓          ↓            ↓           ↓
                Tokens    AST    Type Info    Optimized    Generated
                                              AST          C Code
                                                ↓
                                            Analyzer → Metrics Report
```

### Project Structure

```
minilang_compiler/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs            # Library exports
│   ├── cli.rs            # Command-line interface
│   ├── lexer.rs          # Tokenization
│   ├── parser.rs         # AST construction
│   ├── ast.rs            # AST definitions
│   ├── type_checker.rs   # Semantic analysis
│   ├── symbol_table.rs   # Scope management
│   ├── optimizer.rs      # Optimization passes
│   ├── codegen.rs        # C code generation
│   ├── errors.rs         # Error types
│   ├── wasm.rs           # WebAssembly bindings
│   └── analyzer/         # Static analysis
│       ├── mod.rs        # Orchestrator & display
│       ├── basic.rs      # LOC, statements, params
│       ├── cyclomatic.rs # Cyclomatic complexity
│       ├── cognitive.rs  # Cognitive complexity
│       ├── nesting.rs    # Nesting depth
│       ├── halstead.rs   # Halstead metrics
│       └── fanout.rs     # Fan-out analysis
├── tests/                # Integration tests
├── examples/             # Example programs
├── playground/           # Web playground
│   ├── index.html        
│   ├── script.js         
│   ├── styles.css        
│   ├── pkg/              # WASM build output
│   └── examples/         # Playground examples
├── Cargo.toml
└── README.md
```

## Testing

```bash
# Run all tests
cargo test

# Run analyzer tests (115 tests)
cargo test analyzer

# Run specific test suite
cargo test --test integration_tests

# Run with verbose output
cargo test -- --nocapture
```

## WebAssembly Build

Build the compiler for the web playground:

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM module
wasm-pack build --target web --out-dir web/pkg

# Serve locally
cd playground && python3 -m http.server 8000
```

## Acknowledgments

- **Piston API** — Code execution engine for the web playground
- **Rust** — Incredible compiler infrastructure
- **Logos** — Lightning-fast lexical analysis
- **Miette** — Beautiful error reporting
- **Monaco Editor** — Web-based code editor
- **wasm-pack** — Seamless WebAssembly integration

## Contact

- **Email**: heetabhanushali@gmail.com
- **GitHub**: [@heetabhanushali](https://github.com/heetabhanushali)

---