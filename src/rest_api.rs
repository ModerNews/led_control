pub mod rest_api_mod {
    use crate::commands::commmands::{Commands, Strip};
    use crate::config_utils::configs::{Config, Read, StripConfig};
    use csscolorparser::parse;
    use rocket::form::Form;
    use rocket::http::Status;
    use rocket::request::FromParam;
    use rocket::response::{content, status};
    use rocket::State;
    use tokio::spawn;
    use tokio::sync::Mutex;

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
        strip_config: &State<&'static Mutex<Config>>,
    ) -> status::Custom<content::RawJson<String>> {
        let controllers = strip_config.read_controllers().await;
        let mut strip = Strip::from(controllers.get(name).unwrap());
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
        strip_config: &State<&'static Mutex<Config>>,
    ) -> status::Custom<content::RawJson<String>> {
        let controllers = strip_config.read_controllers().await;
        let mut strip = Strip::from(controllers.get(name).unwrap());
        let status = strip.initialize().await;
        println!("Status: {:?}", status);
        let color = parse(color).unwrap().to_rgba8();
        let command = Commands::SetColor(color[1], color[0], color[2]);
        spawn(async move { strip.execute(&command).await });
        status::Custom(
            Status::Ok,
            content::RawJson(format!("{{\"status\": \"color\", \"name\": \"{}\"}}", name)),
        )
    }

    #[get("/<name>/state")]
    async fn get_state(
        name: &str,
        strip_config: &State<&'static Mutex<Config>>,
    ) -> status::Custom<content::RawJson<String>> {
        let controllers = strip_config.read_controllers().await;
        let mut strip = Strip::from(controllers.get(name).unwrap());
        let _ = strip.initialize().await;
        status::Custom(
            Status::Ok,
            content::RawJson(format!(
                "{{\"status\": \"status\", \"name\": \"{}\", \"state\": {{\"color\": {:?}, \"powered\": {}}}}}",
                name, [strip.color.0, strip.color.1, strip.color.2], strip.powered
            )),
        )
    }


    #[post(
        "/controllers",
        format = "application/x-www-form-urlencoded",
        data = "<target>"
    )]
    async fn add_controller(
        target: Form<StripConfig>,
        strip_config: &State<&'static Mutex<Config>>,
    ) -> status::Custom<content::RawJson<String>> {
        let name = "test".to_string();
        strip_config.lock().await.add_controller(
            &name.to_string(),
            &target.friendly_name,
            &target.ip,
            &target.port,
            &target.is_rgbw,
        );
        status::Custom(
            Status::Ok,
            content::RawJson(format!(
                "{{\"status\": \"create\", \"name\": \"{}\", \"state\": {{{:?}}}}}",
                name, target
            )),
        )
    }

    pub fn rocket(parent_config: &'static Mutex<Config>) -> rocket::Rocket<rocket::Build> {
        rocket::build()
            .manage(parent_config)
            .mount("/", routes![index, on, color, get_state, add_controller])
    }
}
