use rocket::{fairing::{AdHoc, self}, Rocket, Build, error, futures::{StreamExt, future::{join_all, try_join_all}}};
use rocket_db_pools::Database;

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
			Some(db) => {
				match try_join_all(SQL_CREATE_TABLE.split(";").map(|stmt| {
					sqlx::query(stmt).execute(&**db)
				}).collect::<Vec<_>>()).await {
					Ok(res) => {
						res.into_iter().for_each(|res| println!("Query success: affected {} rows", res.rows_affected()));
						Ok(rocket)
					}
					Err(e) => {
						error!("Failed to run query: {}", e);
						Err(rocket)
					}
				}
			}
			None => Err(rocket)
		}
	}
}