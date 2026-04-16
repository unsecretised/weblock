use clap::{Parser, Subcommand};

use crate::proxy::start_proxy;

mod helper;
mod proxy;

#[derive(Debug, Parser, Clone)]
#[command(about, version, long_about = None)]
struct Cli {
    name: String,
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Action {
    Create {
        #[arg(short = 'i', long = "in")]
        inport: u32,
        #[arg(short = 'o', long = "out")]
        outport: u32,
        #[arg(short = 'p', long = "password")]
        password: String,
    },
}

#[cfg(unix)]
fn main() {
    let options = Cli::parse();

    match options.action {
        Action::Create {
            inport,
            outport,
            password,
        } => {
            create_async_proxy(inport, outport, password);
        }
    }
}

pub fn create_async_proxy(inport: u32, outport: u32, password: String) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async { start_proxy(inport, outport, password).await });
}
