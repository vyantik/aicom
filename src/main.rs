#![warn(clippy::all, clippy::pedantic)]

use aicom::{
    cli::{Cli, Commands},
    cli_config::CliConfig,
    gemini::{generate::handle_generate_command, login::is_api_key_valid},
};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli_input = Cli::parse();
    let mut config = CliConfig::load()?;

    match cli_input.command {
        Commands::Login { token } => {
            config.gemini_api_key = Some(token.clone());
            config.save()?;

            if is_api_key_valid(&token).await? {
                println!("✅ Авторизация успешна. API Key сохранен.");
            } else {
                println!("⚠️ Токен сохранен, но похоже, он недействителен.");
            }
        }
        Commands::Generate => {
            handle_generate_command(config).await?;
        }
    }

    Ok(())
}
