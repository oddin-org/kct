pub mod error;
pub mod schema;
pub mod spec;

mod archive;
mod compiler;

use self::error::{Error, Result};
use self::schema::Schema;
use self::spec::Spec;
use compiler::Compiler;
pub use compiler::Release;
use kct_helper::io;
use serde_json::Value;
use std::convert::TryFrom;
use std::path::{Path, PathBuf};

const SCHEMA_FILE: &str = "schema.json";
const SPEC_FILE: &str = "kcp.json";
const EXAMPLE_FILE: &str = "example.json";
const MAIN_FILE: &str = "templates/main.jsonnet";

#[derive(Debug, Clone)]
pub struct Package {
	pub root: PathBuf,
	pub main: PathBuf,
	pub spec: Spec,
	pub schema: Option<Schema>,
	pub example: Option<Value>,
}

impl TryFrom<PathBuf> for Package {
	type Error = Error;

	fn try_from(root: PathBuf) -> Result<Self> {
		let spec = {
			let mut path = root.clone();
			path.push(SPEC_FILE);

			if path.exists() {
				Spec::try_from(path)?
			} else {
				return Err(Error::NoSpec);
			}
		};

		let schema = {
			let mut path = root.clone();
			path.push(SCHEMA_FILE);

			if path.exists() {
				Some(Schema::try_from(path)?)
			} else {
				None
			}
		};

		let example = {
			let mut path = root.clone();
			path.push(EXAMPLE_FILE);

			if path.exists() {
				let value = io::from_file(&path)
					.map_err(|_err| Error::InvalidExample)
					.and_then(|contents| {
						serde_json::from_str(&contents).map_err(|_err| Error::InvalidExample)
					})?;

				Some(value)
			} else {
				None
			}
		};

		let main = {
			let mut path = root.clone();
			path.push(MAIN_FILE);

			if path.exists() {
				path
			} else {
				return Err(Error::NoMain);
			}
		};

		let package = Package {
			root,
			main,
			spec,
			schema,
			example,
		};

		package
			.validate_input(&package.example)
			.map_err(|err| match err {
				Error::InvalidInput => Error::InvalidExample,
				Error::NoInput => Error::NoExample,
				err => err,
			})?;

		Ok(package)
	}
}

/// Methods
impl Package {
	pub fn archive(self, dest: &Path) -> std::result::Result<PathBuf, String> {
		let name = format!("{}_{}", self.spec.name, self.spec.version);
		archive::archive(&name, &self.root, dest)
	}

	pub fn validate_input(&self, input: &Option<Value>) -> Result<()> {
		let (schema, input) = match (&self.schema, &input) {
			(None, None) => return Ok(()),
			(None, Some(_)) => return Err(Error::NoSchema),
			(Some(_), None) => return Err(Error::NoInput),
			(Some(schema), Some(input)) => (schema, input),
		};

		if input.is_object() && schema.validate(input) {
			Ok(())
		} else {
			Err(Error::InvalidInput)
		}
	}

	pub fn compile(self, input: Option<Value>, release: Option<Release>) -> Result<Value> {
		Compiler::new(self).with_release(release).compile(input)
	}
}
