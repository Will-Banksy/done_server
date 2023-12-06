use sqlx::{mysql::MySqlRow, Row};

pub struct User {
	pub id: i32,
	pub name: String,
	pub pass: String,
	pub tasks: Option<Vec<Task>>
}

pub struct Task {
	pub id: i32,
	pub text: String,
	pub user_id: i32
}

impl User {
	pub fn from_row(row: MySqlRow) -> Option<Self> {
		Some(User {
			id: row.try_get("user_id").ok()?,
			name: row.try_get("user_name").ok()?,
			pass: row.try_get("user_pass").ok()?,
			tasks: None
		})
	}
}