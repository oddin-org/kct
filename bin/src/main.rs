mod apply;
mod delete;
mod error;
mod operation;
mod render;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
	version,
	about = "Kubernetes templates without hideous language or context babysitting",
	name = "Kubernetes Configuration Tool"
)]
#[command(
	disable_help_subcommand = true,
	help_expected = true,
	arg_required_else_help = true
)]
pub struct Cli {
	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
pub enum Command {
	#[command(name = "compile", about = "Compiles package into valid manifests")]
	Render(render::Args),
	#[command(
		name = "apply",
		about = "Applies your objects to the currently configured cluster"
	)]
	Apply(apply::Args),
	#[command(
		name = "delete",
		about = "Deletes your objects from the currently configured cluster"
	)]
	Delete(delete::Args),
}

#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();

	let result = match cli.command {
		Command::Render(args) => render::run(args)?,
		Command::Apply(args) => apply::run(args).await?,
		Command::Delete(args) => delete::run(args).await?,
	};

	println!("{result}");

	Ok(())
}
