use rocket::{fairing::{AdHoc, self}, Rocket, Build, error};
use rocket_db_pools::{sqlx, Database};

const SQL_CREATE_TABLE: &'static str = include_str!("../sql/schema.sql");

#[derive(Database)]
#[database("main_db")]
pub struct MainDB(sqlx::MySqlPool);

impl MainDB {
	pub fn stage() -> AdHoc {
		AdHoc::on_ignite("Stage: Create database table", |rocket| async {
			rocket
				.attach(MainDB::init())
				.attach(AdHoc::try_on_ignite("Create database table", MainDB::create_table))
		})
	}

	async fn create_table(rocket: Rocket<Build>) -> fairing::Result {
		match MainDB::fetch(&rocket) {
			Some(db) => match sqlx::query(SQL_CREATE_TABLE).execute(&**db).await {
				Ok(_) => Ok(rocket),
				Err(e) => {
					error!("Failed to create table for database: {}", e);
					Err(rocket)
				}
			}
			None => Err(rocket)
		}
	}
}