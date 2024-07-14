pub mod schema {
    diesel::table! {
            controllers (name) {
                name -> Text,
    friendly_name -> Text,
                ip -> Text,
                port -> Int4,
                is_rgbw -> Bool,
            }
        }
    diesel::table! {
        macros (id) {
            id -> Int4,
            name -> Text,
        }
    }
    diesel::table! {
        actions (id) {
            id -> Int4,
            controller_name -> Text,
            command -> Text,
            macro_id -> Int4,
        }
    }
}

pub mod models {
    use super::schema;
    use diesel::prelude::*;

    #[derive(Identifiable, Queryable, Selectable, Insertable, Debug)]
    #[diesel(table_name = schema::controllers)]
    #[diesel(primary_key(name))]
    pub struct Controller {
        pub name: String,
        pub friendly_name: String,
        pub ip: String,
        pub port: i32,
        pub is_rgbw: bool,
    }

    #[derive(Debug, Identifiable, Queryable, Selectable, Insertable)]
    #[diesel(table_name = schema::macros)]
    pub struct Macro {
        pub id: i32,
        pub name: String,
    }

    #[derive(Debug)]
    pub struct LoadedMacro {
        pub id: i32,
        pub name: String,
        pub actions: Vec<LoadedAction>,
    }

    #[derive(Debug, Identifiable, Queryable, Selectable, Insertable, Associations)]
    #[diesel(table_name = schema::actions)]
    #[diesel(belongs_to(Macro))]
    #[diesel(belongs_to(Controller, foreign_key = controller_name))]
    pub struct Action {
        pub id: i32,
        pub controller_name: String,
        pub command: String,
        pub macro_id: i32,
    }

    #[derive(Debug)]
    pub struct LoadedAction {
        pub id: i32,
        pub controller: Controller,
        pub command: String,
    }
}

pub mod init {
    use diesel::{pg::PgConnection, Connection};

    pub fn establish_connection() -> PgConnection {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }
}

pub mod crud {
    use super::models::*;
    use super::schema::*;
    use diesel::prelude::*;

    pub async fn get_controllers(conn: &mut PgConnection) -> Vec<Controller> {
        controllers::table
            .load::<Controller>(conn)
            .expect("Error loading controllers")
    }

    pub async fn get_macros(conn: &mut PgConnection) -> Vec<LoadedMacro> {
        let target_macros = macros::table
            .load::<Macro>(conn)
            .expect("Error loading macros");
        let loaded_macros = target_macros
            .iter()
            .map(|db_macro: &Macro| {
                let actions = Action::belonging_to(db_macro)
                    .load(conn)
                    .expect("Error loading actions")
                    .iter()
                    .map(|action: &Action| {
                        let controllers = controllers::table
                            .filter(controllers::name.eq(&action.controller_name))
                            .first::<Controller>(conn)
                            .expect("Error loading controllers");
                        LoadedAction {
                            id: action.id,
                            controller: controllers,
                            command: action.command.clone(),
                        }
                    })
                    .collect::<Vec<LoadedAction>>();
                LoadedMacro {
                    id: db_macro.id,
                    name: db_macro.name.clone(),
                    actions,
                }
            })
            .collect::<Vec<LoadedMacro>>();
        loaded_macros
    }

    pub async fn get_actions(conn: &mut PgConnection) -> Vec<Action> {
        actions::table
            .load::<Action>(conn)
            .expect("Error loading actions")
    }
}
