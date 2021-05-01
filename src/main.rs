use std::path::PathBuf;

use argh::FromArgs;
use ion::Section;

// use itertools::Itertools;

#[derive(FromArgs)]
/// Stuff
pub struct Args {
    #[argh(positional)]
    /// target file
    target: PathBuf,

    #[argh(positional)]
    query: String,

    /// prints out the query in debug format instead of running the program
    #[argh(switch, short = 'q')]
    print_debug_query: bool,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    if args.print_debug_query {
        let query = ionql::parse_query(&args.query)?;
        println!("{:#?}", query);
        return Ok(());
    }

    let results = ionql::query_file(args.target, args.query.as_str())?;

    let results = Section {
        dictionary: Default::default(),
        rows: results,
    };

    println!("{}", results);

    Ok(())
}
