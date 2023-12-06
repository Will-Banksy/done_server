pub struct User {
	id: i32,
	name: String,
	pass: String,
	tasks: Option<Vec<Task>>
}

pub struct Task {
	id: i32,
	text: String,
	user_id: i32
}