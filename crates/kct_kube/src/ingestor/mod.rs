mod order;
mod path;

use crate::error::{self, Root as Error};
use crate::Manifest;

use self::order::{Kind, Order, Track, Tracking};

pub use self::path::Filter;

use std::path::PathBuf;

use anyhow::Result;

use serde_json::Value;

pub struct Ingestor {
	filter: Filter,
}

impl Ingestor {
	pub fn new(only: Vec<PathBuf>, except: Vec<PathBuf>) -> Self {
		let filter = Filter { only, except };

		Self { filter }
	}

	pub fn ingest(&self, json: &Value) -> Result<Vec<Manifest>, Error> {
		let mut manifests: Vec<(Tracking, Value)> = vec![];
		let mut walker: Vec<Box<dyn Iterator<Item = (Tracking, &Value)>>> =
			vec![Box::new(vec![(Tracking::default(), json)].into_iter())];

		while let Some(curr) = walker.last_mut() {
			let (tracking, json) = match curr.next() {
				Some(val) => val,
				None => {
					walker.pop();
					continue;
				}
			};

			if Manifest::conforms(json) {
				let order = Order::try_from(json)?;
				let tracking = tracking.ordered(order);
				let kind = Kind::try_from(json)?;
				let tracked = tracking.kinded(kind);

				let path: PathBuf = (&tracked).into();

				if self.filter.pass(&path) {
					manifests.push((tracked, json.to_owned()));
				}
			} else {
				match json {
					Value::Object(map) => {
						let mut members: Vec<(Tracking, &Value)> = Vec::with_capacity(map.len());

						for (k, v) in map {
							if !path::is_valid(k) {
								Err(error::Output::Path(k.to_string()))?;
							} else {
								let track = Track {
									field: k.clone(),
									depth: tracking.depth() + 1,
									order: map.len(),
									kind: None,
								};

								members.push((tracking.track(track), v))
							}
						}

						walker.push(Box::new(members.into_iter()));
					}
					_ => Err(error::Output::NotObject)?,
				}
			}
		}

		manifests.sort_by(|(a, _), (b, _)| a.cmp(b));

		Ok(manifests
			.into_iter()
			.map(|(t, v)| Manifest((&t).into(), v))
			.collect())
	}
}
