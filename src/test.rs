#[cfg(test)]
pub mod tests {
    use serenity::all::{GuildId, RoleId, UserId};
    use surrealdb::engine::remote::ws::Ws;
    use surrealdb::opt::auth::Root;
    use crate::commands::setters::set_joke::Joke;
    use crate::DB;
    use crate::utils::handlers::misc::link_spam_handler::extract_link;
    use crate::commands::setters::{AdminData, ForbiddenRoleData, ForbiddenUserData};
    use crate::utils::misc::debug::UnwrapResult;
    use crate::utils::Warns;

    pub async fn setup() {
        let database_url = dotenvy::var("DATABASE_URL").expect("missing SURREAL_URL");
        let database_password = dotenvy::var("DATABASE_PASSWORD").expect("missing SURREAL_PASSWORD");

        DB.connect::<Ws>(database_url).await.unwrap_or_else(|why| {
            panic!("Could not connect to database: {why}")
        });

        DB.signin(Root {
            username: "root",
            password: &database_password,
        }).await.expect("Could not sign in");

        DB.use_ns("discord-namespace").use_db("discord").await.unwrap();
    }

    #[tokio::test]
    async fn a_extract_link() {
        setup().await;

        let text = "este es un enlace malicioso a https://www.google.com que debería ser detectado";
        let result = extract_link(text);
        assert_eq!(result, Some("https://www.google.com".to_string()));

        b_handle_forbidden_user().await.unwrap();
        c_handle_forbidden_role().await.unwrap();
        d_handle_warn_system().await.unwrap();
        e_handle_joke().await.unwrap();
        f_handle_joke_swtich().await.unwrap();
        g_check_admin_exception().await.unwrap();
    }

    async fn b_handle_forbidden_user() -> UnwrapResult<()> {
        let guild_id = GuildId::new(1_014_327_651_772_674_168);
        let forbidden_user_id = UserId::new(702_527_748_568_121_455);

        let forbidden_user_data = ForbiddenUserData::get_forbidden_user_id(guild_id).await.unwrap().unwrap();
        let database_forbidden_user_id = UserId::new(forbidden_user_data);

        assert_eq!(database_forbidden_user_id, forbidden_user_id);
        println!("test test::tests::b_handle_forbidden_user: ok");

        Ok(())
    }

    async fn c_handle_forbidden_role() -> UnwrapResult<()> {
        let guild_id = GuildId::new(1_014_327_651_772_674_168);
        let forbidden_role_id = "1210266785061019658".to_string();
        let forbidden_role_id_to_u64 = forbidden_role_id.parse::<u64>()?;

        let forbidden_role_data = ForbiddenRoleData::get_role_id(guild_id).await.unwrap().unwrap();
        let database_forbidden_role_id = forbidden_role_data;

        assert_eq!(database_forbidden_role_id, forbidden_role_id_to_u64);
        println!("test test::tests::c_handle_forbidden_role: ok");

        Ok(())
    }

    async fn d_handle_warn_system() -> UnwrapResult<()> {
        let user_id = UserId::new(395_631_548_629_516_298);
        let mut warns = Warns::new(user_id);
        let warns_data = warns.get_warns().await?.unwrap();
        let warns_counter = warns_data.warns;

        // Comprueba si las advertencias coinciden
        warns.get_warns().await?.unwrap();
        if (warns_counter..3).next().is_some() {
            warns.add_warn().await.unwrap();

            if warns_counter >= 3 {
                warns.reset_warns().await?;
                assert_eq!(warns.get_warns().await?.unwrap().warns, 0);
                println!("test test::tests::d_handle_warn_system: ok");
            }
        }

        Ok(())
    }

    async fn e_handle_joke() -> UnwrapResult<()> {
        // Objeto Joke simulando ser obtenido desde la Base de Datos
        let guild_id = GuildId::new(983_473_640_387_518_504);
        let joke = Joke::get_joke_target_id(guild_id).await?;

        // user_id obtenido desde Discord
        let user_id: u64 = 1_076_623_900_697_448_478;

        // Comparamos el target del objeto Joke con el user_id hardcodeado
        assert_eq!(joke, user_id);
        println!("test test::tests::e_handle_joke ok");

        Ok(())
    }

    async fn f_handle_joke_swtich() -> UnwrapResult<()> {
        let database_password = dotenvy::var("DATABASE_PASSWORD").expect("missing SURREAL_PASSWORD");
        DB.signin(Root {
            username: "root",
            password: &database_password,
        }).await.expect("Could not sign in");

        // Objeto Joke simulando ser obtenido desde la Base de Datos
        let guild_id = GuildId::new(983_473_640_387_518_504);
        let joke = Joke::get_joke_status(guild_id).await?;

        let mut joke = Joke::new(guild_id.to_string(), joke, String::new());

        if joke.is_active {
            joke.switch(false).await?;
            assert!(!joke.is_active);
            println!("test test::tests::true f_handle_joke_swtich: ok");
        } else {
            joke.switch(true).await?;
            assert!(joke.is_active);
            println!("test test::tests::false f_handle_joke_swtich: ok");
        }

        Ok(())
    }

    async fn g_check_admin_exception() -> UnwrapResult<()> {
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
        println!("test test::tests::role_1 g_check_admin_exception: ok");

        assert_eq!(database_role_id_2, admin_role_id_2);
        println!("test test::tests::role_2 g_check_admin_exception: ok");

        Ok(())
    }
}