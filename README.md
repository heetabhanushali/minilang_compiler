# MiniLang Compiler 🚀

[![Build Status](https://img.shields.io/badge/build-passing-success)](https://github.com/yourusername/minilang)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)
[![WebAssembly](https://img.shields.io/badge/wasm-ready-purple)](https://webassembly.org/)

A **statically-typed, compiled programming language** built entirely in Rust, featuring a unique syntax with Pascal-style logical operators (`AND`/`OR`/`NOT`) and a complete compiler pipeline from source to native executable.

## 🎯 Live Demo

**[Try MiniLang in your browser →](https://your-demo-url.com)**

Experience the interactive playground with real-time compilation, AST visualization, and optimization statistics.

## ✨ Features

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

## 🚀 Quick Start

### Web Playground

Visit the [online playground](https://your-demo-url.com) to start coding immediately, or run locally:

```bash
cd web
python3 -m http.server 8000
# Visit http://localhost:8000
```

### Command Line

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/heetabhanushali/minilang.git
cd minilang_compiler
cargo build --release

# Run your first program
echo 'func main() { display "Hello, MiniLang!"; }' > hello.mini
cargo run --release -- run hello.mini
```

## 📝 Language Syntax

### Hello World
```minilang
func main() {
    display "Hello, World!";
}
```

### Variables and Types
```minilang
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
```minilang
func factorial(n: int) -> int {
    if n <= 1 {
        send 1;
    }
    send n * factorial(n - 1);
}

func main() {
    # Loops
    for i = 1; i <= 5; i = i + 1 {
        if i % 2 == 0 AND i != 4 {
            display "Even: ", i;
        }
    }
    
    # Pattern: display sends output, send returns values
    let result: int = factorial(5);
    display "5! = ", result;
}
```

## 🛠️ CLI Usage

### Commands

| Command | Description |
|---------|-------------|
| `minilang compile <file>` | Compile to executable |
| `minilang run <file>` | Compile and run immediately |
| `minilang check <file>` | Type-check without compiling |
| `minilang ast <file>` | Display Abstract Syntax Tree |
| `minilang tokens <file>` | Display token stream |
| `minilang stats <file>` | Show optimization statistics |
| `minilang clean` | Remove generated files |

### Options

| Option | Description |
|--------|-------------|
| `-o, --output <name>` | Output executable name |
| `-O, --opt <level>` | Optimization level (0-2) |
| `-d, --detail` | Show compilation steps |
| `--keep-c` | Keep intermediate C file |

### Examples

```bash
# Compile with optimizations
minilang compile program.mini -O 2

# Check for errors without compiling
minilang check program.mini

# View the AST
minilang ast program.mini

# See optimization statistics
minilang stats program.mini --time
```

## 🏗️ Architecture

```
Source (.mini) → Lexer → Parser → Type Checker → Optimizer → Code Gen → C Code → GCC → Executable
                   ↓        ↓          ↓            ↓           ↓
                Tokens    AST    Type Info    Optimized    Generated
                                              AST          C Code
```

### Project Structure

```
minilang_compiler/
├── src/
│   ├── main.rs           
│   ├── lib.rs            
│   ├── lexer.rs          
│   ├── parser.rs         
│   ├── ast.rs            
│   ├── type_checker.rs   
│   ├── optimizer.rs      
│   ├── codegen.rs        
│   ├── symbol_table.rs   
│   ├── errors.rs         
│   └── cli.rs            
├── tests/                # Integration tests
├── web/
│   ├── index.html        
│   ├── script.js         
│   ├── styles.css        
│   └── examples/         # Example programs
├── Cargo.toml
└── README.md
```

## 🔬 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --test integration_tests

# Run with verbose output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## 🌐 WebAssembly Build

Build the compiler for the web playground:

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM module
wasm-pack build --target web --out-dir web/pkg

# Serve locally
cd web && python3 -m http.server
```

## 🙏 Acknowledgments

- **Rust** for the incredible compiler infrastructure
- **Logos** for lightning-fast lexical analysis
- **Miette** for beautiful error reporting
- **Monaco Editor** for the web-based code editor
- **wasm-pack** for seamless WebAssembly integration

## 📧 Contact

- Email: heetabhaushali@gmail.com

---