use std::path::PathBuf;

use argh::FromArgs;
use ion::Section;
use ionql::config::Config;

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

    /// overrides the default config location
    #[argh(option)]
    config: Option<PathBuf>,
}

const CONFIG_SUBDIR: &str = "ionql";
const DEFAULT_CONFIG_FILENAME: &str = "config.json";

fn default_config_location() -> PathBuf {
    let base_dirs =
        directories::BaseDirs::new().expect("Failed to create base dirs");

    base_dirs
        .config_dir()
        .join(CONFIG_SUBDIR)
        .join(DEFAULT_CONFIG_FILENAME)
}

fn main() -> anyhow::Result<()> {
    let args: Args = argh::from_env();

    if args.print_debug_query {
        let query = ionql::parse_query(&args.query)?;
        println!("{:#?}", query);
        return Ok(());
    }

    let config_location = if let Some(loc) = args.config {
        loc
    } else {
        default_config_location()
    };

    let config = Config::load_or_create_default(&config_location)?;

    let results = ionql::query_file(args.target, args.query.as_str(), &config)?;

    let results = Section {
        dictionary: Default::default(),
        rows: results,
    };

    println!("{}", results);

    Ok(())
}
