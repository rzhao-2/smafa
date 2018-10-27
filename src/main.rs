extern crate smafa;

extern crate log;
use log::LogLevelFilter;
extern crate env_logger;
use env_logger::LogBuilder;

extern crate clap;
use clap::*;

use std::env;

fn main(){
    let mut app = build_cli();
    let matches = app.clone().get_matches();

    match matches.subcommand_name() {
        Some("query") => {
            let m = matches.subcommand_matches("query").unwrap();
            let db_root = m.value_of("DB").unwrap();
            let query_fasta = m.value_of("QUERY_FASTA").unwrap();
            let max_divergence = value_t!(m.value_of("divergence"), u32).unwrap_or(5);
            set_log_level(m);
            smafa::query(db_root, query_fasta, max_divergence, &mut std::io::stdout());
        },
        Some("makedb") => {
            let m = matches.subcommand_matches("makedb").unwrap();
            let db_fasta = m.value_of("DB_FASTA").unwrap();
            let db_root = m.value_of("DB").unwrap();
            set_log_level(m);
            smafa::makedb(db_root, db_fasta);
        },
        Some("cluster") => {
            let m = matches.subcommand_matches("cluster").unwrap();
            let query_fasta = m.value_of("FASTA").unwrap();
            let max_divergence = value_t!(m.value_of("divergence"), u32).unwrap_or(5);
            set_log_level(m);
            if m.is_present("fragment-method") {
                smafa::fragment_clusterer::cluster_by_fragment(
                    query_fasta, max_divergence as u8, &mut std::io::stdout());
            } else {
                smafa::cluster(query_fasta, max_divergence, &mut std::io::stdout());
            }
        },
        _ => {
            app.print_help().unwrap();
            println!();
        }
    }
}

fn set_log_level(matches: &clap::ArgMatches) {
    let mut log_level = LogLevelFilter::Info;
    if matches.is_present("verbose") {
        log_level = LogLevelFilter::Debug;
    }
    if matches.is_present("quiet") {
        log_level = LogLevelFilter::Error;
    }
    let mut builder = LogBuilder::new();
    builder.filter(None, log_level);
    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }
    builder.init().unwrap();
}

fn build_cli() -> App<'static, 'static> {
    let makedb_args: &'static str = "<DB_FASTA>  'Subject sequences to search against'
                       <DB>        'Output DB filename root'

                      -v, --verbose       'Print extra debug logging information'
                      -q, --quiet         'Unless there is an error, do not print logging information'";
    let query_args: &'static str = "<DB>           'Output from makedb'
                      <QUERY_FASTA> 'Query sequences to search with'
                      -d, --divergence=[INTEGER] 'Maximum number of mismatches in reported hits [default: 5]'

                      -v, --verbose       'Print extra debug logging information'
                      -q, --quiet         'Unless there is an error, do not print logging information'";
    let cluster_args: &'static str = "<FASTA> 'Sequences to cluster'
                      -d, --divergence=[INTEGER] 'Maximum number of mismatches in reported hits [default: 5]'

                      -v, --verbose       'Print extra debug logging information'
                      -q, --quiet         'Unless there is an error, do not print logging information'";

    return App::new("smafa")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ben J. Woodcroft <benjwoodcroft near gmail.com>")
        .about("Read aligner for small pre-aligned sequences")
        .args_from_usage("-v, --verbose       'Print extra debug logging information'
             -q, --quiet         'Unless there is an error, do not print logging information'")
        .subcommand(
            SubCommand::with_name("makedb")
                .about("Generate a searchable database")
                .args_from_usage(&makedb_args))
        .subcommand(
            SubCommand::with_name("query")
                .about("Search a database")
                .args_from_usage(&query_args))
        .subcommand(
            SubCommand::with_name("cluster")
                .about("Cluster sequences greedily, preferring sequences towards front of file")
                .arg(Arg::with_name("fragment-method")
                     .long("fragment-method")
                     .help("Use the 'fragment' method for clustering"))
                .args_from_usage(&cluster_args));
}

