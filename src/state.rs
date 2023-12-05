use std::{collections::BTreeMap, sync::RwLock};

pub struct Tasks {
	pub tasks: RwLock<BTreeMap<String, String>>
}