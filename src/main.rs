use std::error::Error;

use clap::value_parser;
use server::{server::Server, service::Service};

#[cfg(not(target_os = "wasi"))]
fn main() -> Result<(), Box<dyn Error>> {
    let args = clap::Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("GIT_HASH"))
        .arg(
            clap::Arg::new("port")
                .short('p')
                .long("port")
                .value_name("port")
                .help("Server port")
                .value_parser(value_parser!(u16))
                .required(true),
        )
        .arg(
            clap::Arg::new("log")
                .short('l')
                .long("log")
                .value_name("level")
                .help("Define log level")
                .value_parser(clap::builder::PossibleValuesParser::new([
                    "error", "warn", "info", "debug", "trace",
                ]))
                .default_value("trace")
                .required(false),
        )
        .get_matches();

    let port: u16 = *args.get_one("port").expect("`port` is required");

    if args.contains_id("log") {
        let log_level: &String = args.get_one::<String>("log").expect("`log` is required");

        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, log_level),
        );
    }

    let mut server = Server::new(port)?;
    Service::run(&mut server)
}

#[cfg(target_os = "wasi")]
fn main() {
    panic!("can't bind to an address with wasi")
}
