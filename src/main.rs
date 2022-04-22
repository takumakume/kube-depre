use file::FileSystem;
use utils::{Finder, Output, Scrape};
mod cluster;
mod file;
mod utils;
use crate::cluster::Cluster;
use crate::utils::init_logger;
use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Sunset {
    #[clap(long = "target-version", short = 't')]
    target_version: Option<String>,
    /// Output format table, junit, csv
    #[clap(long = "output", short = 'o', arg_enum,default_value_t = Output::Table)]
    output: Output,
    #[clap(long, short)]
    kubeconfig: Option<String>,
    /// Scrape the cluster for deprecated apis,
    #[clap(long, short)]
    file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    debug: usize,
}

impl Sunset {
    // if there is a mention of -d in the args, it will be scraping the directory else default will be cluster
    fn check_scrape_type(&self) -> Scrape {
        match &self.file {
            Some(d) => Scrape::Dir(d.to_string(), "Filename"),
            None => Scrape::Cluster("Namespace"),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Sunset::parse();
    // You can check the value provided by positional arguments, or option arguments
    let version: String = if let Some(version) = &cli.target_version {
        version.to_string()
    } else {
        "1.16".to_string()
    };

    match cli.debug {
        1 => {
            std::env::set_var("RUST_LOG", "info,kube=debug");
        }
        _ => std::env::set_var("RUST_LOG", "info,kube=info"),
    }

    init_logger();

    match cli.check_scrape_type() {
        Scrape::Cluster(col_replace) => {
            let c = Cluster::new(version).await?;
            let x = utils::VecTableDetails(c.find_deprecated_api().await?);
            match cli.output {
                Output::Csv => {
                    x.generate_csv(col_replace)?;
                }
                Output::Junit => {
                    println!("Junit");
                }
                Output::Table => {
                    x.generate_table(col_replace)?;
                }
            }
        }
        Scrape::Dir(loc, col_replace) => {
            let c = FileSystem::new(loc, version).await?;
            let x = utils::VecTableDetails(c.find_deprecated_api().await?);
            match cli.output {
                Output::Csv => {
                    x.generate_csv(col_replace)?;
                }
                Output::Junit => {
                    println!("Junit");
                }
                Output::Table => {
                    x.generate_table(col_replace)?;
                }
            }
        }
    };
    Ok(())
}
