use clap::ArgMatches;
use clap::{App, Arg, SubCommand};
use helper::io;
use package::{Package, Release};
use serde_json::Value;
use std::path::PathBuf;

pub fn command() -> App<'static, 'static> {
	SubCommand::with_name("compile")
		.about("compiles the package into valid k8s objects")
		.arg(
			Arg::with_name("package")
				.help("Target to run the subcommand onto")
				.index(1)
				.required(true),
		)
		.arg(
			Arg::with_name("values")
				.short("f")
				.long("values")
				.help("Sets the values for the package")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("release")
				.long("release")
				.help("Scope your package within a release")
				.takes_value(true),
		)
}

// TODO: Can't we use Box<dyn Display> for error since all our errors implement
// Display?
pub fn run(matches: &ArgMatches) -> Result<String, String> {
	let values_from: Option<PathBuf> = matches.value_of("values").map(PathBuf::from);
	let values = parse_values(&values_from)?;

	let package_from: PathBuf = matches.value_of("package").map(PathBuf::from).unwrap();
	let package = Package::from_path(package_from).map_err(|err| err.to_string())?;

	let release = matches.value_of("release").map(|name| Release {
		name: String::from(name),
	});

	let rendered = package
		.compile(values, release)
		.map_err(|err| err.to_string())?;

	let objects = kube::find(&rendered).map_err(|err| err.to_string())?;
	let to_apply = kube::glue(&objects);

	Ok(to_apply.to_string())
}

fn parse_values(path: &Option<PathBuf>) -> Result<Option<Value>, String> {
	match path {
		None => Ok(None),
		Some(path) => {
			let contents = if path == &PathBuf::from("-") {
				io::from_stdin().map_err(|err| err.to_string())?
			} else {
				io::from_file(path).map_err(|err| err.to_string())?
			};

			let file = path.to_str().unwrap();
			let parsed: Value = serde_json::from_str(&contents)
				.map_err(|_err| format!("Unable to parse {}", file))?;

			Ok(Some(parsed))
		}
	}
}
