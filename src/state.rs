use std::{collections::{BTreeMap, HashMap}, sync::RwLock};

use crate::security;

pub struct Tasks {
	pub tasks: RwLock<BTreeMap<String, String>>
}

pub struct SessionManager {
	/// Map of session cookie values to user ids
	pub sessions: RwLock<HashMap<u128, i32>>
}

impl SessionManager {
	pub fn new() -> Self {
		SessionManager {
			sessions: RwLock::new(HashMap::new())
		}
	}

	/// Creates and returns a session id as a string, and keeps track of that session id mapped to the supplied user id
	pub fn start_session(&self, user_id: i32) -> String {
		let sessid = security::generate_session_cookie();
		self.sessions.write().unwrap().insert(sessid, user_id);
		(format!("{:#018x}", sessid)[2..]).to_string()
	}

	/// If the session id is well formed and is mapped to a user id, that user id is returned
	pub fn query_session(&self, sessid: &str) -> Option<i32> {
		let sessid = u128::from_str_radix(sessid, 16).ok()?;
		self.sessions.read().unwrap().get(&sessid).map(|i| *i)
	}

	pub fn stop_session(&self, sessid: &str) {
		if let Ok(sessid) = u128::from_str_radix(sessid, 16) {
			self.sessions.write().unwrap().remove(&sessid);
		}
	}
}

#[cfg(test)]
#[test]
fn test_hex_encode_decode() {
	let sessid = security::generate_session_cookie();
	let sessid_str = (format!("{:#018x}", sessid)[2..]).to_string();
	let sessid_decoded = u128::from_str_radix(&sessid_str, 16).unwrap();
	assert_eq!(sessid, sessid_decoded);
}