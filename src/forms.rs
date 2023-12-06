use rocket::FromForm;

#[derive(FromForm, Debug)]
pub struct TaskForm<'a> {
	pub id: &'a str,
	pub task: &'a str,
}

#[derive(FromForm)]
pub struct SignupForm<'a> {
	pub username: &'a str,
	pub password: &'a str,
}

#[derive(FromForm)]
pub struct LoginForm<'a> {
	pub username: &'a str,
	pub password: &'a str,
}