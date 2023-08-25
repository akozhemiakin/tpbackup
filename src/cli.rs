use std::{fs, path::PathBuf};

use crate::{client::Client, utils, writer::SyncWriter};
use clap::{Args, Parser, Subcommand};
use tokio::{
    fs::{create_dir_all, File},
    io::stdout,
};
use tracing::debug;

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
    #[arg(long, env = "TPB_HOST")]
    host: String,

    /// TargetProcess user name
    #[arg(short, long, env = "TPB_USER")]
    user: String,

    /// TargetProcess password
    #[arg(short, long, env = "TPB_PASSWORD")]
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

    /// Outputs result into stdout as a sequence of json objects
    /// (one for each resource) without separators between them.
    #[arg(short, long, conflicts_with = "compress", conflicts_with = "out_dir")]
    stdout: bool,
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

        let is_stdout = cli.stdout;

        if is_stdout {
            debug!("start backup in stdout");

            client
                .backup_all(
                    1,
                    false,
                    cli.resources
                        .map(|v| v.split(',').map(|v| v.trim().to_string()).collect())
                        .unwrap_or_else(|| {
                            utils::RESOURCES.iter().map(|v| v.to_string()).collect()
                        }),
                    |_| async { SyncWriter::new(stdout()) },
                )
                .await;
        } else {
            debug!("start backup in files");

            // Write somewhere in temp dir for tarballs or to some local dir otherwise
            let dir = if cli.compress {
                let mut p = std::env::temp_dir();

                p.push("tpbackup");

                let _ = fs::remove_dir_all(&p);

                p
            } else {
                cli.out_dir.unwrap_or(PathBuf::from(r"./out"))
            };

            create_dir_all(dir.clone()).await.unwrap();

            let p = dir.clone();

            client
                .backup_all(
                    5,
                    !cli.no_progress,
                    cli.resources
                        .map(|v| v.split(',').map(|v| v.trim().to_string()).collect())
                        .unwrap_or_else(|| {
                            utils::RESOURCES.iter().map(|v| v.to_string()).collect()
                        }),
                    move |resource| {
                        let mut p = p.clone();

                        p.push(format!("{resource}.json"));

                        async { File::create(p).await.unwrap() }
                    },
                )
                .await;

            // Compress if needed
            if cli.compress {
                debug!("compressing results");

                let out = cli.out.unwrap_or_else(|| {
                    PathBuf::from(format!("tpbackup_{}.tar.gz", utils::yyyymmddhhmm()))
                });

                utils::compress_dir_to_tar_gz(&dir, &out).unwrap();

                // Remove contents of a temp directory
                let _ = fs::remove_dir_all(dir);
            }
        }
    }
}
