#[cfg(test)]
pub mod tests {
    use serenity::all::{GuildId, RoleId, UserId};
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Root;
    use tokio::task;
    use crate::commands::setters::set_joke::Joke;
    use crate::DB;
    use crate::utils::handlers::sent_messages::extract_link;
    use crate::commands::setters::{AdminData, ForbiddenRoleData, ForbiddenUserData};
    use crate::commands::setters::set_to_blacklist::BlackListData;
    use crate::utils::Warns;

    pub async fn setup() {
        let database_url = dotenvy::var("DATABASE_URL").expect("missing SURREAL_URL");
        let database_password = dotenvy::var("DATABASE_PASSWORD").expect("missing SURREAL_PASSWORD");

        match DB.connect::<Ws>(database_url).await {
            Ok(()) => (),
            Err(why) => {
                if why.to_string() != "Already connected" {
                    eprintln!("Could not connect to database: {why}");
                }
            },
        };

        DB.signin(Root {
            username: "root",
            password: &database_password,
        }).await.expect("Could not sign in");

        DB.use_ns("discord-namespace").use_db("discord").await.unwrap();

        task::yield_now().await; // Necesario para que la conexión a la base de datos se establezca correctamente
    }

    #[test]
    fn a_extract_link() {
        let text = "este es un enlace malicioso a https://www.google.com que debería ser detectado";
        let result = extract_link(text);
        assert_eq!(result, Some("https://www.google.com".to_string()));
    }

    #[tokio::test]
    async fn b_handle_forbidden_user() {
        setup().await;

        let guild_id = GuildId::new(1_014_327_651_772_674_168);
        let forbidden_user_id = UserId::new(1_203_715_036_061_892_628);

        let forbidden_user_data = ForbiddenUserData::get_forbidden_user_id(guild_id).await.unwrap().unwrap();
        let database_forbidden_user_id = UserId::new(forbidden_user_data);

        assert_eq!(database_forbidden_user_id, forbidden_user_id);

        task::yield_now().await;
    }

    #[tokio::test]
    async fn c_handle_forbidden_role() {
        setup().await;
        let guild_id = GuildId::new(1_014_327_651_772_674_168);
        let forbidden_role_id = "1210266785061019658".to_string();
        let forbidden_role_id_to_u64 = forbidden_role_id.parse::<u64>().unwrap();

        let forbidden_role_data = ForbiddenRoleData::get_role_id(guild_id).await.unwrap().unwrap();
        let database_forbidden_role_id = forbidden_role_data;

        assert_eq!(database_forbidden_role_id, forbidden_role_id_to_u64);

        task::yield_now().await;
    }

    #[tokio::test]
    async fn d_handle_warn_system() {
        setup().await;

        let user_id = UserId::new(395_631_548_629_516_298);
        let mut warns = Warns::new(user_id);
        let warns_data = warns.get_warns().await.unwrap().unwrap();
        let warns_counter = warns_data.warns;

        // Comprueba si las advertencias coinciden
        warns.get_warns().await.unwrap().unwrap();
        if (warns_counter..3).next().is_some() {
            warns.add_warn().await.unwrap();

            if warns_counter >= 3 {
                warns.reset_warns().await.unwrap();
                assert_eq!(warns.get_warns().await.unwrap().unwrap().warns, 0);
            }
        }

        task::yield_now().await;
    }

    #[tokio::test]
    async fn e_handle_joke() {
        setup().await;

        // Objeto Joke simulando ser obtenido desde la Base de Datos
        let guild_id = GuildId::new(983_473_640_387_518_504);
        let joke = Joke::get_joke_target_id(guild_id).await.unwrap();

        // user_id obtenido desde Discord
        let user_id: u64 = 1_076_623_900_697_448_478;

        // Comparamos el target del objeto Joke con el user_id hardcodeado
        assert_eq!(joke, user_id);

        task::yield_now().await;
    }

    #[tokio::test]
    async fn f_handle_joke_swtich() {
        setup().await;

        let database_password = dotenvy::var("DATABASE_PASSWORD").expect("missing SURREAL_PASSWORD");
        DB.signin(Root {
            username: "root",
            password: &database_password,
        }).await.expect("Could not sign in");

        // Objeto Joke simulando ser obtenido desde la Base de Datos
        let guild_id = GuildId::new(983_473_640_387_518_504);
        let joke = Joke::get_joke_status(guild_id).await.unwrap();

        let mut joke = Joke::new(guild_id.to_string(), joke, String::new());

        if joke.is_active {
            joke.switch(false).await.unwrap();
            assert!(!joke.is_active);
        } else {
            joke.switch(true).await.unwrap();
            assert!(joke.is_active);
        }

        task::yield_now().await;
    }

    #[tokio::test]
    async fn g_check_admin_exception() {
        setup().await;

        let guild_id = GuildId::new(1_014_327_651_772_674_168);

        // El rol de administrador simulado desde la API de Discord
        let admin_role_1_string = "1020156814152712222".to_string();
        let admin_role_id_to_u64 = admin_role_1_string.parse::<u64>().unwrap();
        let admin_role_id = RoleId::new(admin_role_id_to_u64);

        let get_role_id = AdminData::get_admin_role(guild_id).await.unwrap();
        let role_to_u64 = get_role_id.unwrap().parse::<u64>().unwrap();
        let database_role_id_1 = RoleId::new(role_to_u64);

        let admin_role_2_string = "1196523947177545858".to_string();
        let admin_role_id_to_u64 = admin_role_2_string.parse::<u64>().unwrap();
        let admin_role_id_2 = RoleId::new(admin_role_id_to_u64);

        let get_role_id_2 = AdminData::get_admin_role_2(guild_id).await.unwrap();
        let role_to_u64 = get_role_id_2.unwrap().parse::<u64>().unwrap();
        let database_role_id_2 = RoleId::new(role_to_u64);

        assert_eq!(database_role_id_1, admin_role_id);
        assert_eq!(database_role_id_2, admin_role_id_2);

        task::yield_now().await;
    }

    // no hace falta usar namespace
    #[tokio::test]
    async fn h_get_blacklist_link() {
        setup().await;

        let guild_id = GuildId::new(1_014_327_651_772_674_168);
        let link = "GET YOUR ADBOE PHOTOSHOP FREE NOW: https://www.youtube.com/watch?v=N-gcKsjVMp0 @everyone @here".to_string();
        let extracted_link = extract_link(&link).unwrap();
        let result = BlackListData::get_blacklist_link(guild_id, extracted_link.clone()).await.unwrap();
        assert_eq!(result, extracted_link);

        task::yield_now().await;
    }
}