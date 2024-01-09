pub use crate::log_handle;

pub struct Data {
    pub client: reqwest::Client,
}

pub type CommandResult = Result<(), Error>;
pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn err_handler(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Error al iniciar el Bot: {error:?}"),
        poise::FrameworkError::Command { error, ctx} => {
            log_handle!("Error en comando `{}` : {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error al manejar el error: {e}")
            }
        }
    }
}

#[macro_export]
macro_rules! log_handle {
    ($($arg:tt)*) => {
        {
            use std::io::Write;

            // Obtener la hora actual y formatearla
            let current_time = chrono::Local::now();
            let error_msg = format!("[{}] Error: {}\n", current_time.format("%Y-%m-%d %H:%M:%S"), format!($($arg)*));

            // Imprimir el mensaje de error en la consola
            eprintln!("{error_msg}");

            // Guardar el mensaje de error en el archivo de logs
            let log_result = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("log.txt");

            if let Err(err) = &log_result {
                eprintln!("Error al abrir el archivo de logs: {err}");
            }

            // Si se pudo abrir el archivo de log, escribir el mensaje de error en el archivo
            if let Ok(mut file) = log_result {
                if let Err(err) = write!(file, "{error_msg}") {
                    eprintln!("Error al escribir en el archivo de logs: {err}");
                }
            }
        }
    };
}