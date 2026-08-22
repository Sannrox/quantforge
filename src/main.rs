use std::path::PathBuf;

use clap::{Parser, Subcommand};
use quantforge::error::AppError;
use quantforge::host::{ServeOptions, parse_bind, serve};
use quantforge::store::Store;

#[derive(Parser)]
#[command(
    name = "quantforge",
    about = "Local-first long-term investor research workbench"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        #[arg(long, default_value_t = 4176)]
        port: u16,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        testdata: Option<PathBuf>,
        #[arg(long)]
        web_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();
    match cli.command {
        Command::Serve {
            bind,
            port,
            db,
            testdata,
            web_dir,
        } => {
            let bind = parse_bind(&bind)?;
            let testdata = testdata.unwrap_or_else(default_testdata);
            serve(ServeOptions {
                bind,
                port,
                db: db.unwrap_or_else(Store::default_path),
                testdata,
                web_dir: web_dir.or_else(default_web_dir),
            })
            .await
        }
    }
}

fn default_testdata() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

fn default_web_dir() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("web")
        .join("dist");
    path.exists().then_some(path)
}
