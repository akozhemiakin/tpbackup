use std::{fs, path::PathBuf};

use crate::{client::Client, utils};
use clap::{Args, Parser, Subcommand};

/// This utility cycles through all or some of Targetprocess resources and backs
/// up each type of resource into a separate JSON file. It also provides
/// an option to package these JSON files into a single tarball.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Performs a backup
    Backup(BackupArgs),

    /// Outputs a default list of resources
    Resources,
}

#[derive(Debug, Args)]
struct BackupArgs {
    /// Host name of TargetProcess instance
    ///
    /// Example: myinstance.tpondemand.com
    host: String,

    /// TargetProcess user name
    #[arg(short, long)]
    user: String,

    /// TargetProcess password
    #[arg(short, long)]
    password: String,

    /// Hide progress bars
    #[arg(long)]
    no_progress: bool,

    /// Compress output files into single tar.gz archive
    #[arg(long)]
    compress: bool,

    /// Target dir to put resulting JSON files
    #[arg(long, conflicts_with = "compress")]
    out_dir: Option<PathBuf>,

    /// Target file name to put tar.gz contents
    #[arg(short, long, conflicts_with = "out_dir", requires = "compress")]
    out: Option<PathBuf>,

    /// Comma separated resources list
    ///
    /// E.g., UserStories,Bugs,Features
    ///
    /// To get full list of resources included by default run `tpbackup resources`
    #[arg(short, long)]
    resources: Option<String>,
}

impl Cli {
    pub async fn run() {
        let cli = Cli::parse();

        match cli.commands {
            Commands::Backup(args) => {
                Self::backup(args).await;
            }
            Commands::Resources => {
                Self::resources();
            }
        }
    }

    fn resources() {
        println!("{}", utils::RESOURCES.join("\n"))
    }

    async fn backup(cli: BackupArgs) {
        let client = Client::new(cli.host, cli.user, cli.password);

        // Write somewhere in temp dir for tarballs or to some local dir otherwise
        let dir = if cli.compress {
            let mut p = std::env::temp_dir();

            p.push("tpbackup");

            let _ = fs::remove_dir_all(&p);

            p
        } else {
            cli.out_dir.unwrap_or(PathBuf::from(r"./out"))
        };

        client
            .backup_all(
                5,
                !cli.no_progress,
                dir.clone(),
                cli.resources
                    .map(|v| v.split(',').map(|v| v.trim().to_string()).collect())
                    .unwrap_or_else(|| utils::RESOURCES.iter().map(|v| v.to_string()).collect()),
            )
            .await;

        // Compress if needed
        if cli.compress {
            let out = cli.out.unwrap_or_else(|| {
                PathBuf::from(format!("tpbackup_{}.tar.gz", utils::yyyymmddhhmm()))
            });

            utils::compress_dir_to_tar_gz(&dir, &out).unwrap();

            // Remove contents of a temp directory
            let _ = fs::remove_dir_all(dir);
        }
    }
}
