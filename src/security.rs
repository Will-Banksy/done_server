use argon2::{password_hash::{SaltString, rand_core::OsRng}, Argon2, PasswordHasher, PasswordHash, PasswordVerifier};

pub fn password_hash(pass: &str) -> String {
	let salt = SaltString::generate(&mut OsRng);

	let argon2 = Argon2::default();

	argon2.hash_password(pass.as_bytes(), &salt).unwrap().to_string()
}

pub fn password_verify(pass: &str, pass_hash: &str) -> bool {
	let parsed_hash = PasswordHash::new(&pass_hash).unwrap();

	Argon2::default().verify_password(pass.as_bytes(), &parsed_hash).is_ok()
}

pub fn generate_session_cookie() -> u128 {
	uuid::Uuid::new_v4().as_u128()
}