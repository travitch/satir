use std::path::PathBuf;
use structopt::StructOpt;

use satirlib;

#[derive(Debug,StructOpt)]
#[structopt(version = "1.0", author = "Tristan Ravitch")]
struct Options {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf
}

fn main() -> anyhow::Result<()> {
    let opts = Options::from_args();
    let contents = std::fs::read_to_string(opts.input)?;
    let dimacs = satirlib::satir::parse::dimacs::parse_dimacs(&contents)?;
    let res = satirlib::satir::dpll::solve(dimacs.clauses, dimacs.next_var);
    match res {
        satirlib::satir::core::Result::Unsat => print!("unsat\n"),
        satirlib::satir::core::Result::Sat => print!("sat\n")
    };

    Ok(())
}
