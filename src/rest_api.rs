pub mod rest_api_mod {
    use crate::commands::commmands::{Commands, Strip};
    use crate::config_utils::configs::Config;
    use csscolorparser::parse;
    use rocket::http::Status;
    use rocket::request::FromParam;
    use rocket::response::{content, status};
    use rocket::State;
    use tokio::spawn;

    #[get("/")]
    fn index() -> status::Custom<content::RawJson<&'static str>> {
        status::Custom(Status::Ok, content::RawJson("{\"status\": \"ok\"}"))
    }

    impl<'r> FromParam<'r> for Commands {
        type Error = &'r str;

        fn from_param(param: &'r str) -> Result<Self, Self::Error> {
            match param {
                "on" => Ok(Commands::On),
                "off" => Ok(Commands::Off),
                _ => Err(param),
            }
        }
    }

    #[get("/<name>/<command>")]
    async fn on(
        name: &str,
        command: Commands,
        strip_config: &State<Config>,
    ) -> status::Custom<content::RawJson<String>> {
        let mut strip = Strip::from(strip_config.controllers.get(name).unwrap());
        let status = strip.initialize().await;
        println!("Status: {:?}", status);
        spawn(async move { strip.execute(&command).await });
        status::Custom(
            Status::Ok,
            content::RawJson(format!("{{\"status\": \"on\", \"name\": \"{}\"}}", name)),
        )
    }

    #[get("/<name>/set/<color>")]
    async fn color(
        name: &str,
        color: &str,
        strip_config: &State<Config>,
    ) -> status::Custom<content::RawJson<String>> {
        let mut strip = Strip::from(strip_config.controllers.get(name).unwrap());
        let status = strip.initialize().await;
        println!("Status: {:?}", status);
        let color = parse(color).unwrap().to_rgba8();
        let command = if strip.is_rgbw {
            Commands::SetColor(color[1], color[0], color[2])
        } else {
            Commands::SetColor(color[0], color[1], color[2])
        };
        spawn(async move { strip.execute(&command).await });
        status::Custom(
            Status::Ok,
            content::RawJson(format!("{{\"status\": \"color\", \"name\": \"{}\"}}", name)),
        )
    }

    #[get("/<name>/state")]
    async fn get_state(
        name: &str,
        strip_config: &State<Config>,
    ) -> status::Custom<content::RawJson<String>> {
        let mut strip = Strip::from(strip_config.controllers.get(name).unwrap());
        let _ = strip.initialize().await;
        status::Custom(
            Status::Ok,
            content::RawJson(format!(
                "{{\"status\": \"status\", \"name\": \"{}\", \"state\": {{\"color\": {:?}, \"powered\": {}}}}}",
                name, [strip.color.0, strip.color.1, strip.color.2], strip.powered
            )),
        )
    }

    pub fn rocket(parent_config: Config) -> rocket::Rocket<rocket::Build> {
        rocket::build()
            .manage(parent_config)
            .mount("/", routes![index, on, color, get_state])
    }
}
