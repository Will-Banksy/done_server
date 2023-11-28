use rocket::{launch, routes, get, fs::{FileServer, Options}};
use rocket_dyn_templates::{Template, context};

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {})
}

#[get("/login")]
fn login() -> Template {
    Template::render("login", context! {})
}

#[get("/tasks")]
fn tasks() -> Template {
    Template::render("tasks", context! {})
}

#[get("/signup")]
fn signup() -> Template {
    Template::render("signup", context! {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, tasks, signup, login])
        .mount("/assets/", FileServer::new("assets", Options::None))
        .attach(Template::fairing())
}