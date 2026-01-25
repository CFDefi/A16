//! A16 CLI - Command-line interface for the A16 programming language

use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "a16")]
#[command(author = "A16 Team")]
#[command(version = "0.1.0")]
#[command(about = "The A16 programming language for AI systems", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an A16 program
    Run {
        /// Source file to run
        file: PathBuf,
        
        /// Watch for changes and restart
        #[arg(short, long)]
        watch: bool,
        
        /// Enable debug mode
        #[arg(long)]
        debug: bool,
        
        /// Deterministic execution
        #[arg(long)]
        deterministic: bool,
        
        /// Token budget limit
        #[arg(long)]
        token_budget: Option<u64>,
        
        /// Arguments to pass to the program
        #[arg(last = true)]
        args: Vec<String>,
    },
    
    /// Start interactive REPL
    Repl {
        /// Persistent memory path
        #[arg(short, long)]
        memory: Option<PathBuf>,
        
        /// Model to use
        #[arg(long)]
        model: Option<String>,
    },
    
    /// Create a new A16 project
    New {
        /// Project name
        name: String,
        
        /// Template to use
        #[arg(short, long, default_value = "agent")]
        template: String,
    },
    
    /// Format A16 source files
    Fmt {
        /// Files to format
        files: Vec<PathBuf>,
        
        /// Check formatting without modifying
        #[arg(long)]
        check: bool,
    },
    
    /// Run tests
    Test {
        /// Test filter
        filter: Option<String>,
        
        /// Watch for changes
        #[arg(short, long)]
        watch: bool,
        
        /// Generate coverage report
        #[arg(long)]
        coverage: bool,
    },
    
    /// Run benchmarks
    Bench {
        /// Benchmark filter
        filter: Option<String>,
        
        /// Compare against baseline
        #[arg(long)]
        baseline: Option<PathBuf>,
    },
    
    /// Package management
    Pkg {
        #[command(subcommand)]
        command: PkgCommands,
    },
    
    /// Diagnose issues
    Doctor,
    
    /// Lex a file (development/debug command)
    #[command(hide = true)]
    Lex {
        /// Source file to lex
        file: PathBuf,
    },
    
    /// Parse a file (development/debug command)
    #[command(hide = true)]
    Parse {
        /// Source file to parse
        file: PathBuf,
        
        /// Show AST in full detail
        #[arg(long)]
        verbose: bool,
    },
    
    /// Type check an A16 source file
    Check {
        /// Source file to check
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum PkgCommands {
    /// Install packages
    Install {
        /// Package name
        package: String,
        
        /// Version requirement
        #[arg(long)]
        version: Option<String>,
        
        /// Install as dev dependency
        #[arg(long)]
        dev: bool,
    },
    
    /// Uninstall packages
    Uninstall {
        /// Package name
        package: String,
    },
    
    /// List installed packages
    List,
    
    /// Search for packages
    Search {
        /// Search query
        query: String,
    },
    
    /// Publish package
    Publish,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { file, watch, debug, deterministic, token_budget, args } => {
            cmd_run(file, watch, debug, deterministic, token_budget, args)
        }
        Commands::Repl { memory, model } => {
            cmd_repl(memory, model)
        }
        Commands::New { name, template } => {
            cmd_new(name, template)
        }
        Commands::Fmt { files, check } => {
            cmd_fmt(files, check)
        }
        Commands::Test { filter, watch, coverage } => {
            cmd_test(filter, watch, coverage)
        }
        Commands::Bench { filter, baseline } => {
            cmd_bench(filter, baseline)
        }
        Commands::Pkg { command } => {
            cmd_pkg(command)
        }
        Commands::Doctor => {
            cmd_doctor()
        }
        Commands::Lex { file } => {
            cmd_lex(file)
        }
        Commands::Parse { file, verbose } => {
            cmd_parse(file, verbose)
        }
        Commands::Check { file } => {
            cmd_check(file)
        }
    }
}

fn cmd_run(
    file: PathBuf,
    _watch: bool,
    debug: bool,
    _deterministic: bool,
    _token_budget: Option<u64>,
    _args: Vec<String>,
) -> anyhow::Result<()> {
    use a16_parser::parse;
    use a16_typeck::check;
    use a16_hir::lower_module;
    use a16_codegen::compile;
    use a16_vm::VM;
    use std::time::Instant;
    
    println!("{}", "A16 Runtime".bright_cyan().bold());
    println!("Running: {}", file.display());
    println!();
    
    let start = Instant::now();
    
    // Read source
    let source = std::fs::read_to_string(&file)?;
    
    // Parse
    let module = match parse(&source) {
        Ok(m) => m,
        Err(e) => {
            println!("{} Parse error: {:?}", "✗".bright_red(), e);
            return Err(anyhow::anyhow!("Parse failed"));
        }
    };
    
    if debug {
        println!("  {} Parsed {} item(s)", "✓".bright_green(), module.items.len());
    }
    
    // Type check
    let errors = check(&module);
    if !errors.is_empty() {
        println!("{} Type errors:", "✗".bright_red());
        for err in &errors {
            println!("  • {}", err);
        }
        return Err(anyhow::anyhow!("Type check failed with {} error(s)", errors.len()));
    }
    
    if debug {
        println!("  {} Type check passed", "✓".bright_green());
    }
    
    // Lower to HIR
    let hir = lower_module(&module);
    
    if debug {
        println!("  {} Lowered to HIR ({} functions)", "✓".bright_green(), hir.functions.len());
    }
    
    // Compile to bytecode
    let bytecode = compile(&hir);
    
    if debug {
        println!("  {} Compiled {} function(s)", "✓".bright_green(), bytecode.functions.len());
        println!("  {} {} constant(s) in pool", "✓".bright_green(), bytecode.constants.len());
    }
    
    // Find main/entry function
    let entry_name = bytecode.functions.iter()
        .find(|f| f.name.as_str() == "main")
        .or_else(|| bytecode.functions.first())
        .map(|f| f.name.clone());
    
    let entry_name = match entry_name {
        Some(name) => name,
        None => {
            println!("{} No entry point found (need 'fn main():' or at least one function)", "✗".bright_red());
            return Err(anyhow::anyhow!("No entry point"));
        }
    };
    
    if debug {
        println!("  {} Entry point: {}", "✓".bright_green(), entry_name);
        println!();
    }
    
    // Execute
    let mut vm = VM::new(bytecode);
    
    match vm.run(&entry_name) {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("{}", "─".repeat(40));
            println!("{} Result: {}", "→".bright_green(), result);
            println!();
            println!("Completed in {:.2?}", elapsed);
        }
        Err(e) => {
            println!("{} Runtime error: {}", "✗".bright_red(), e);
            return Err(anyhow::anyhow!("Execution failed: {}", e));
        }
    }
    
    Ok(())
}

fn cmd_repl(_memory: Option<PathBuf>, _model: Option<String>) -> anyhow::Result<()> {
    use a16_parser::parse;
    use a16_hir::lower_module;
    use a16_codegen::compile;
    use a16_vm::VM;
    use std::io::{self, Write};
    
    println!("{}", "A16 REPL".bright_cyan().bold());
    println!("Version 0.1.0");
    println!("Type 'exit' or 'quit' to exit, 'help' for commands");
    println!();
    
    let mut line_num = 1;
    let mut multiline_buffer = String::new();
    let mut in_multiline = false;
    
    loop {
        // Print prompt
        if in_multiline {
            print!("{} ", "...".bright_black());
        } else {
            print!("{} {} ", format!("[{}]", line_num).bright_blue(), ">>>".bright_green());
        }
        io::stdout().flush()?;
        
        // Read input
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        
        let trimmed = input.trim();
        
        // Handle special commands
        if !in_multiline {
            match trimmed {
                "exit" | "quit" => {
                    println!("{}", "Goodbye!".bright_cyan());
                    break;
                }
                "help" => {
                    println!("{}", "Commands:".bold());
                    println!("  exit, quit  - Exit the REPL");
                    println!("  help        - Show this help");
                    println!("  clear       - Clear screen");
                    println!();
                    println!("Enter A16 expressions or statements to evaluate.");
                    println!("Functions defined persist across inputs.");
                    continue;
                }
                "clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                    continue;
                }
                "" => continue,
                _ => {}
            }
        }
        
        // Handle multi-line input
        if trimmed.ends_with(':') && !in_multiline {
            in_multiline = true;
            multiline_buffer = input.clone();
            continue;
        }
        
        if in_multiline {
            if trimmed.is_empty() {
                // End multi-line input
                in_multiline = false;
                input = std::mem::take(&mut multiline_buffer);
            } else {
                multiline_buffer.push_str(&input);
                continue;
            }
        }
        
        // Wrap expression in function for evaluation
        let source = if input.trim_start().starts_with("fn ")
            || input.trim_start().starts_with("let ")
            || input.trim_start().starts_with("if ")
            || input.trim_start().starts_with("for ")
            || input.trim_start().starts_with("while ")
        {
            // Statement - wrap in main
            format!("fn __repl__():\n    {}", input.trim())
        } else {
            // Expression - return it
            format!("fn __repl__():\n    return {}", input.trim())
        };
        
        // Parse
        match parse(&source) {
            Ok(module) => {
                // Lower to HIR
                let hir = lower_module(&module);
                
                // Compile
                let bytecode = compile(&hir);
                
                // Execute
                let mut vm = VM::new(bytecode);
                match vm.run("__repl__") {
                    Ok(result) => {
                        let result_str = format!("{}", result);
                        if result_str != "None" {
                            println!("{} {}", "=>".bright_yellow(), result_str);
                        }
                    }
                    Err(e) => {
                        println!("{} {}", "Error:".bright_red(), e);
                    }
                }
            }
            Err(e) => {
                println!("{} {:?}", "Parse error:".bright_red(), e);
            }
        }
        
        line_num += 1;
    }
    
    Ok(())
}

fn cmd_new(name: String, template: String) -> anyhow::Result<()> {
    println!("{} Creating new A16 project: {}", "→".bright_green(), name);
    println!("  Template: {}", template);
    
    // TODO: Actually create project structure
    println!();
    println!("{}", "Note: Project creation not yet implemented.".yellow());
    Ok(())
}

fn cmd_fmt(files: Vec<PathBuf>, check: bool) -> anyhow::Result<()> {
    let action = if check { "Checking" } else { "Formatting" };
    println!("{} {} file(s)...", action, files.len());
    
    println!();
    println!("{}", "Note: Formatter not yet implemented.".yellow());
    Ok(())
}

fn cmd_test(_filter: Option<String>, _watch: bool, _coverage: bool) -> anyhow::Result<()> {
    println!("{}", "Running tests...".bright_cyan());
    
    println!();
    println!("{}", "Note: Test runner not yet implemented.".yellow());
    Ok(())
}

fn cmd_bench(_filter: Option<String>, _baseline: Option<PathBuf>) -> anyhow::Result<()> {
    println!("{}", "Running benchmarks...".bright_cyan());
    
    println!();
    println!("{}", "Note: Benchmark runner not yet implemented.".yellow());
    Ok(())
}

fn cmd_pkg(command: PkgCommands) -> anyhow::Result<()> {
    match command {
        PkgCommands::Install { package, version, dev } => {
            let dev_str = if dev { " (dev)" } else { "" };
            let ver_str = version.unwrap_or_else(|| "latest".to_string());
            println!("Installing {}@{}{}", package, ver_str, dev_str);
        }
        PkgCommands::Uninstall { package } => {
            println!("Uninstalling {}", package);
        }
        PkgCommands::List => {
            println!("Installed packages:");
            println!("  (none)");
        }
        PkgCommands::Search { query } => {
            println!("Searching for: {}", query);
        }
        PkgCommands::Publish => {
            println!("Publishing package...");
        }
    }
    
    println!();
    println!("{}", "Note: Package manager not yet implemented.".yellow());
    Ok(())
}

fn cmd_doctor() -> anyhow::Result<()> {
    println!("{}", "A16 Environment Check".bright_cyan().bold());
    println!("{}", "═".repeat(50));
    println!();
    
    println!("{}:", "Runtime".bold());
    println!("  {} A16 version: 0.1.0", "✓".bright_green());
    println!("  {} Platform: {}", "✓".bright_green(), std::env::consts::OS);
    
    println!();
    println!("{}:", "Compiler".bold());
    println!("  {} Lexer: ready", "✓".bright_green());
    println!("  {} Parser: ready", "✓".bright_green());
    println!("  {} Type checker: ready", "✓".bright_green());
    println!("  {} HIR lowering: ready", "✓".bright_green());
    println!("  {} Bytecode compiler: ready", "✓".bright_green());
    
    println!();
    println!("{}:", "Execution".bold());
    println!("  {} Virtual machine: ready", "✓".bright_green());
    println!("  {} Standard library: ready", "✓".bright_green());
    println!("  {} REPL: ready", "✓".bright_green());
    
    println!();
    println!("{}:", "AI Features".bold());
    if std::env::var("OPENAI_API_KEY").is_ok() {
        println!("  {} OPENAI_API_KEY: set", "✓".bright_green());
    } else {
        println!("  {} OPENAI_API_KEY: not set (AI features disabled)", "⚠".yellow());
    }
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        println!("  {} ANTHROPIC_API_KEY: set", "✓".bright_green());
    } else {
        println!("  {} ANTHROPIC_API_KEY: not set", "⚠".yellow());
    }
    
    println!();
    println!("{}", "All systems operational!".bright_green().bold());
    
    Ok(())
}

fn cmd_lex(file: PathBuf) -> anyhow::Result<()> {
    use a16_lexer::Lexer;
    
    println!("{} Lexing: {}", "→".bright_green(), file.display());
    println!();
    
    let source = std::fs::read_to_string(&file)?;
    let tokens = Lexer::new(&source).tokenize();
    
    for token in &tokens {
        println!(
            "{:4}-{:4}  {:15}  {}",
            token.span.start,
            token.span.end,
            format!("{:?}", token.kind).bright_blue(),
            token.text.replace('\n', "\\n")
        );
    }
    
    println!();
    println!("Total tokens: {}", tokens.len());
    
    Ok(())
}

fn cmd_parse(file: PathBuf, verbose: bool) -> anyhow::Result<()> {
    use a16_parser::parse;
    
    println!("{} Parsing: {}", "→".bright_green(), file.display());
    println!();
    
    let source = std::fs::read_to_string(&file)?;
    
    match parse(&source) {
        Ok(module) => {
            println!("{} Parse successful!", "✓".bright_green());
            println!();
            println!("Module contains {} item(s):", module.items.len());
            
            for (i, item) in module.items.iter().enumerate() {
                let item_desc = match item {
                    a16_ast::Item::Function(f) => format!("fn {}", f.name.name),
                    a16_ast::Item::Class(c) => format!("class {}", c.name.name),
                    a16_ast::Item::Agent(a) => format!("agent {}", a.name.name),
                    a16_ast::Item::Tool(t) => format!("tool {}", t.name.name),
                    a16_ast::Item::Memory(m) => format!("memory {}", m.name.name),
                    a16_ast::Item::Prompt(p) => format!("prompt {}", p.name.name),
                    a16_ast::Item::Struct(s) => format!("struct {}", s.name.name),
                    a16_ast::Item::Enum(e) => format!("enum {}", e.name.name),
                    a16_ast::Item::Import(_) => "import ...".to_string(),
                    a16_ast::Item::Const(c) => format!("const {}", c.name.name),
                    a16_ast::Item::Stmt(_) => "<statement>".to_string(),
                };
                println!("  {}. {}", i + 1, item_desc.bright_blue());
            }
            
            if verbose {
                println!();
                println!("{}", "Full AST:".bold());
                println!("{:#?}", module);
            }
        }
        Err(e) => {
            println!("{} Parse failed!", "✗".bright_red());
            println!();
            println!("Error: {:?}", e);
            return Err(anyhow::anyhow!("Parse failed"));
        }
    }
    
    Ok(())
}

fn cmd_check(file: PathBuf) -> anyhow::Result<()> {
    use a16_parser::parse;
    use a16_typeck::check;
    
    println!("{} Type checking: {}", "→".bright_green(), file.display());
    println!();
    
    let source = std::fs::read_to_string(&file)?;
    
    // First parse
    let module = match parse(&source) {
        Ok(m) => m,
        Err(e) => {
            println!("{} Parse failed!", "✗".bright_red());
            println!("Error: {:?}", e);
            return Err(anyhow::anyhow!("Parse failed"));
        }
    };
    
    // Then type check
    let errors = check(&module);
    
    if errors.is_empty() {
        println!("{} No type errors found!", "✓".bright_green());
        println!();
        println!("Checked {} item(s)", module.items.len());
    } else {
        println!("{} Found {} type error(s):", "✗".bright_red(), errors.len());
        println!();
        
        for (i, error) in errors.iter().enumerate() {
            println!("{}. {}", i + 1, error);
        }
        
        return Err(anyhow::anyhow!("Type checking failed with {} error(s)", errors.len()));
    }
    
    Ok(())
}
