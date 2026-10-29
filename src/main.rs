use live_server::{
    cli::{Cli, CliCommand},
    control, server,
};

#[tokio::main]
async fn main() {
    let command = match Cli::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("Live Server CLI error: {error}\n\n{}", Cli::help());
            std::process::exit(2);
        }
    };

    let result = match command {
        CliCommand::Help => {
            println!("{}", Cli::help());
            Ok(())
        }
        CliCommand::Version => {
            println!("{}", live_server::VERSION);
            Ok(())
        }
        CliCommand::Mcp => live_server::mcp::run_stdio(),
        CliCommand::Start(options) => server::run(*options).await,
        CliCommand::Restart(options) => server::restart(*options).await,
        CliCommand::Stop(options) => control::stop(&options.root).map(|message| {
            println!("{message}");
        }),
        CliCommand::Open(options) => {
            control::open(&options.root, options.browser.as_ref()).map(|message| {
                println!("{message}");
            })
        }
        CliCommand::Status(options) => control::status(&options.root).map(|_| ()),
    };

    if let Err(error) = result {
        eprintln!("Live Server error: {error}");
        std::process::exit(1);
    }
}
