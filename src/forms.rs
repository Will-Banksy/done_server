use rocket::FromForm;

#[derive(FromForm, Debug)]
pub struct TaskForm<'a> {
	pub id: &'a str,
	pub task: &'a str,
}