pub mod rest_api {
    use rocket::http::Status;
    use rocket::response::{content, status};

    #[get("/")]
    fn index() -> status::Custom<content::RawJson<&'static str>> {
        status::Custom(Status::Ok, content::RawJson("{\"status\": \"ok\"}"))
    }

    pub fn rocket() -> rocket::Rocket<rocket::Build> {
        rocket::build().mount("/", routes![index])
    }
}
