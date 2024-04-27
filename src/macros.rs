pub mod led_macro {
    use std::time::Duration;

    use crate::commands::commmands::{Commands, Strip};
    use crate::config_utils::configs::{Config, MacroConfig, StripConfig};
    // use std::time::
    use cron::Schedule;
    use serde::{Deserialize, Serialize};
    use tokio::{spawn, time};

    #[derive(Clone, Debug)]
    pub struct Macro {
        pub name: String,
        pub actions: Vec<(StripConfig, Commands)>,
    }

    #[derive(Clone)]
    pub struct Event {
        pub name: String,
        pub trigger: EventTriggers,
        pub action: Macro,
    }

    #[derive(Clone)]
    pub enum EventTriggers {
        Cron(Schedule),
        Startup,
        Shutdown,
    }

    /*     impl From<&MacroConfig, &Config> for Macro {
        pub fn from(config: &MacroConfig, parent: &Config) -> Self {
            Macro {
                name: config.name,
                actions: (
                    parent.controllers.get(config.actions.0),
                    Commands::from(config.actions.1),
                ),
            }
        }
    } */

    impl Macro {
        pub fn new(config: &MacroConfig, parent: &Config) -> Self {
            let mut actions = Vec::new();
            for action in config.actions.iter() {
                actions.push((
                    parent.controllers.get(&action.0).unwrap().clone(),
                    Commands::from(&action.1),
                ));
            }
            Macro {
                name: config.name.clone(),
                actions,
            }
        }

        pub async fn run(&self) {
            for (target, command) in self.actions.iter() {
                let mut target_strip = Strip::from(target);
                let _ = target_strip.initialize().await;
                let task = command.clone();
                spawn(async move { target_strip.execute(&task).await });
            }
        }
    }

    // TODO: Implement Redis macro dispatcher
    pub async fn native_event_dispatcher(macros: Vec<Event>) {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let execution_moment = chrono::Utc::now();
            tokio::spawn(macro_executor(macros.clone(), execution_moment));
        }
    }

    async fn macro_executor(macros: Vec<Event>, moment: chrono::DateTime<chrono::Utc>) {
        for macro_ in macros.iter() {
            match &macro_.trigger {
                EventTriggers::Cron(schedule) => {
                    if schedule.includes(moment) {
                        macro_.action.run().await;
                    }
                }
                _ => {} // TODO: Other triggers should be implemented elsewhere
            }
        }
    }
}
