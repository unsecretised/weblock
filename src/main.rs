use std::{fs::File, process::exit};

use clap::{Parser, Subcommand};
use daemonize::Daemonize;

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
        #[arg(short = 'd', long = "detach", default_value_t = false)]
        detach: bool,
    },
    Delete,
}

fn get_env(env_name: &str) -> String {
    match std::env::var(env_name) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Could not find the: {env_name} env variable: {e}");
            exit(1)
        }
    }
}

#[cfg(unix)]
fn main() {
    let options = Cli::parse();

    match options.action {
        Action::Create {
            inport,
            outport,
            password,
            detach,
        } => {
            if detach {
                let stdout = File::create(format!("/tmp/{}-weblock.out", options.name)).unwrap();
                let stderr = File::create(format!("/tmp/{}-weblock.err", options.name)).unwrap();
                let daemon = Daemonize::new()
                    .pid_file(format!("/tmp/{}-weblock.pid", options.name))
                    .stdout(stdout)
                    .stderr(stderr)
                    .user(get_env("USER").as_str());

                daemon.start().expect("Could not daemonize process");
            }
            create_async_proxy(inport, outport, password);
        }
        _ => {}
    }
}

pub fn create_async_proxy(inport: u32, outport: u32, password: String) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async { start_proxy(inport, outport, password).await });
}
