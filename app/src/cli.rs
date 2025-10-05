use std::{convert::TryInto, path::PathBuf, str::FromStr};

use clap::{arg, command, Parser, Subcommand};
use error_stack::{Result, ResultExt};
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::{
    enums::{Brightness, Direction, Effects},
    manager::{
        self,
        custom_effect::CustomEffect,
        profile::{self, Profile},
        ManagerCreationError,
    },
};

#[macro_export]
macro_rules! clap_value_parser {
    ($v: expr, $e: ty) => {{
        use clap::builder::TypedValueParser;
        clap::builder::PossibleValuesParser::new($v).map(|s| s.parse::<$e>().unwrap())
    }};
}

#[derive(Parser)]
#[command(
    author,
    version,
    long_about = None,
    name = "Legion Keyboard Control",
    arg_required_else_help = true,
    rename_all = "camelCase",
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(

    // rename_all = "PascalCase",
)]
enum Commands {
    /// Use an effect from the built-in set
    Set {
        /// The effect to be set
        #[arg(short, long, value_enum, value_parser, rename_all = "PascalCase")]
        effect: Effects,

        /// List of 4 RGB triplets. Example: 255,0,0,255,255,0,0,0,255,255,128,0
        #[arg(short, long, default_value = "0,0,0,0,0,0,0,0,0,0,0,0", value_parser = parse_colors)]
        colors: Option<[u8; 12]>,

        /// The brightness of the effect [possible values: Low, High]
        #[arg(short, long, default_value = "Low", value_parser)]
        brightness: Brightness,

        /// The speed of the effect
        #[arg(short, long, default_value_t = 1, value_parser = clap_value_parser!(["1","2","3","4","5"], u8))]
        speed: u8,

        /// The direction of the effect (If applicable)
        #[arg(short, long, value_enum)]
        direction: Option<Direction>,

        /// A filename to save the effect at
        #[arg(long, value_enum)]
        save: Option<PathBuf>,
    },

    /// List all the available effects
    List,

    /// Load a profile from a file
    LoadProfile {
        #[arg(short, long)]
        path: PathBuf,
    },

    /// Load a custom effect from a file
    CustomEffect {
        #[arg(short, long)]
        path: PathBuf,
    },
}

fn parse_colors(arg: &str) -> std::result::Result<[u8; 12], String> {
    fn input_err<E>(_e: E) -> String {
        "Invalid input, please check you used the correct format for the colors".to_string()
    }

    let vec: std::result::Result<Vec<u8>, <u8 as FromStr>::Err> = arg.split(',').map(str::parse::<u8>).collect();
    let vec = vec.map_err(input_err);

    match vec {
        Ok(vec) => {
            let vec: std::result::Result<[u8; 12], Vec<u8>> = vec.try_into();

            vec.map_err(input_err)
        }
        Err(err) => Err(err),
    }
}

pub enum CliOutput {
    /// CLI arguments were passed
    Cli(OutputType),
}

/// What instruction was received through the CLI
#[derive(Clone)]
pub enum OutputType {
    Profile(Profile),
    Custom(CustomEffect),
    Exit,
}

#[derive(Debug, Error)]
#[error("There was an error while executing the CLI")]
pub struct CliError;

pub fn try_cli() -> Result<(), CliError> {
    let output_type = parse_cli()?;

    match output_type {
        CliOutput::Cli(output_type) => handle_cli_output(output_type),
    }
}

fn handle_cli_output(output_type: OutputType) -> Result<(), CliError> {
    let manager_result = manager::EffectManager::new(manager::OperationMode::Cli);
    let instance_not_unique = manager_result.as_ref().err().is_some_and(|err| &ManagerCreationError::InstanceAlreadyRunning == err.current_context());

    if matches!(output_type, OutputType::Profile(..) | OutputType::Custom(..)) && instance_not_unique {
        println!("Another instance of the program is already running, please close it before starting a new one.");
        return Ok(());
    }

    let mut effect_manager = manager_result.change_context(CliError)?;

    let command_result = match output_type {
        OutputType::Profile(profile) => {
            effect_manager.set_profile(profile);
            Ok(())
        }
        OutputType::Custom(effect) => {
            effect_manager.custom_effect(effect);
            Ok(())
        }
        OutputType::Exit => Ok(()),
    };

    effect_manager.shutdown();
    command_result
}

fn parse_cli() -> Result<CliOutput, CliError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set {
            effect,
            colors,
            brightness,
            speed,
            direction,
            save,
        } => {
            let direction = direction.unwrap_or_default();
            let rgb_array = if effect.takes_color_array() {
                colors.unwrap_or_else(|| {
                    println!("This effect requires specifying the colors to use.");
                    std::process::exit(0);
                })
            } else {
                [0; 12]
            };

            let mut profile = Profile {
                name: None,
                rgb_zones: profile::arr_to_zones(rgb_array),
                effect,
                direction,
                speed,
                brightness,
            };

            if let Some(filename) = save {
                profile.save_profile(&filename).expect("Failed to save.");
            }

            Ok(CliOutput::Cli(OutputType::Profile(profile)))
        }
        Commands::List => {
            println!("List of available effects:");
            for (i, effect) in Effects::iter().enumerate() {
                println!("{}. {effect}", i + 1);
            }
            Ok(CliOutput::Cli(OutputType::Exit))
        }

        Commands::LoadProfile { path } => {
            let profile = Profile::load_profile(&path).change_context(CliError)?;
            Ok(CliOutput::Cli(OutputType::Profile(profile)))
        }

        Commands::CustomEffect { path } => {
            let effect = CustomEffect::from_file(&path).change_context(CliError)?;
            Ok(CliOutput::Cli(OutputType::Custom(effect)))
        }
    }
}
