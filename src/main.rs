use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs)]
/// Stuff
pub struct Args {
    #[argh(positional)]
    /// target file
    target: PathBuf,

    #[argh(positional)]
    query: String,
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    let results = ionql::query_file(args.target, args.query.as_str())?;

    println!("{:#?}", results);

    Ok(())
}
