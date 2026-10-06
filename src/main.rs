//! The binary is a thin shell around `gridrs::cli::run`; all behaviour lives in the library
//! so that it can be tested. This file is outside every task's write scope.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = gridrs::cli::run(&args, &mut std::io::stdout(), &mut std::io::stderr());
    std::process::exit(code);
}
