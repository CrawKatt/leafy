<p align="center"><img src="https://cdn.discordapp.com/avatars/1207370166028083230/73e2a023b6797922d02e3b6f6bcb7109.png?size=256" alt="project-image"></p>

<h1 align="center" id="title">Leafy</h1>

Leafy es un Bot de Discord que posee funciones de moderación, comandos de música, sistema anti tag (@), comandos de entretenimiento y más.

## Comandos disponibles:
| Comando                 | Categoría       | Tipo                 | Descripción                                                                                                                                                                                      |
|-------------------------|-----------------|----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| ping                    | Info            | Prefix/Slash Command | Muestra la latencia del Bot                                                                                                                                                                      |
| help                    | Info            | Prefix/Slash command | Muestra un menú de ayuda con los comandos del Bot                                                                                                                                                |
| ask                     | Info            | Prefix/Slash Command | Habla con la IA del Bot                                                                                                                                                                          |
| sst                     | Entretenimiento | Prefix Command       | Crea una pseudocaptura de pantalla de un mensaje al que respondas                                                                                                                                |
| pride                   | Entretenimiento | Prefix Command       | Aplica un Overlay Arcoíris a la foto de perfil del usuario                                                                                                                                       |
| furry                   | Entretenimiento | Prefix Command       | Envía una imágen de broma Furry con la foto de perfil del usuario                                                                                                                                |
| set_admins              | Moderación      | Prefix/Slash Command | Establece hasta dos roles que el Bot reconocerá como administrador                                                                                                                               |
| set_log_channel         | Moderación      | Prefix/Slash Command | Establece el canal de Logs del Bot                                                                                                                                                               |
| set_ooc_channel         | Moderación      | Prefix/Slash Command | Establece el canal de Fuera de Contexto                                                                                                                                                          |
| set_warn_message        | Moderación      | Prefix/Slash Command | Establece el mensaje personalizado de advertencia                                                                                                                                                |
| set_timeout_timer       | Moderación      | Prefix/Slash Command | Establece el tiempo que el Bot aplicará como sanción de aislamiento                                                                                                                              |
| set_forbidden_user      | Moderación      | Prefix/Slash Command | Establece el usuario que no está permitido mencionar (hacer @ o responder mensajes con `@`                                                                                                       |
| set_forbidden_role      | Moderación      | Prefix/Slash Command | Establece el rol que no está permitido mencionar (hacer @ o responder mensajes con `@`                                                                                                           |
| set_welcome_message     | Moderación      | Prefix/Slash Command | Establece el mensaje de Bienvenida del Bot para los miembros nuevos                                                                                                                              |
| set_welcome_channel     | Moderación      | Prefix/Slash Command | Establece el canal de Bienvenida del Bot para los miembros nuevos                                                                                                                                |
| set_time_out_message    | Moderación      | Prefix/Slash Command | Establece el mensaje de timeout del Bot cuando se aplique a un usuario                                                                                                                           |
| set_forbidden_exception | Moderación      | Prefix/Slash Command | Establece una excepción para el usuario no mencionable si este la solicita para permitir el uso de `@`                                                                                           |
| set_exception_channel   | Moderación      | Prefix/Slash Command | Establece un canal de excepción para establecer excepciones dinámicas para el usuario no mencionable (se permite el uso de `@` siempre y cuando el usuario esté continuamente en la conversación |
| get_admins              | Moderación      | Prefix/Slash Command | Obtiene los administradores establecidos                                                                                                                                                         |
| get_log_channel         | Moderación      | Prefix/Slash Command | Obtiene el canal de Logs establecido                                                                                                                                                             |
| get_ooc_channel         | Moderación      | Prefix/Slash Command | Obtiene el canal de Fuera de Contexto establecido                                                                                                                                                |
| get_timeout_timer       | Moderación      | Prefix/Slash Command | Obtiene el tiempo de timeout establecido                                                                                                                                                         |
| get_forbidden_user      | Moderación      | Prefix/Slash Command | Obtiene el usuario no mencionable establecido                                                                                                                                                    |
| get_forbidden_role      | Moderación      | Prefix/Slash Command | Obtiene el rol no mencionable establecido                                                                                                                                                        |
| get_welcome_channel     | Moderación      | Prefix/Slash Command | Obtiene el canal de bienvenidas establecido                                                                                                                                                      |
| get_exception_channel   | Moderación      | Prefix/Slash Command | Obtiene el canal de excepciones dinámicas establecido                                                                                                                                            |
| get_forbidden_exception | Moderación      | Prefix/Slash Command | Obtiene el usuario que ha solicitado una excepción                                                                                                                                               |
| rust                    | Aprendizaje     | Prefix/Slash Command | Enseña ejemplos de código en Rust para aprender a programar                                                                                                                                      |

## 🛠️ Dependencias:

<p>
    1. Rust
</p>

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

<p>
    2. SurrealDB
</p>

```
curl -sSf https://install.surrealdb.com | sh
```

<p>
    3. Opus - Códec de audio que utiliza Discord. audiopus utilizará los binarios libopus instalados si están 
    disponibles a través de pkgconf en Linux/MacOS, de lo contrario tendrá que instalar cmake para construir opus 
    desde el código fuente. Este es siempre el caso en Windows. Para sistemas Unix, puede instalar la biblioteca con
    <code>apt install libopus-dev</code> en Ubuntu o <code>pacman -S opus</code> en Arch Linux. Si no la tienes 
    instalada, se creará para ti. Sin embargo, necesitarás un compilador de C y las autotools de GNU instaladas.
    De nuevo, estos pueden ser instalados con <code>apt install build-essential autoconf automake libtool m4</code>
    en Ubuntu o <code>pacman -S base-devel</code> en Arch Linux.
</p>

<p>
    4. ffmpeg - ffmpeg es una colección de software libre que maneja audio, video y otros archivos multimedia y 
    bibliotecas de software. Puedes instalarlo con <code>apt install ffmpeg</code> en Ubuntu o <code>pacman -S ffmpeg</code>
    en Arch Linux.
</p>

<p>
    5. yt-dlp / youtube-dl / (forks similares) - Herramienta de descarga de audio/vídeo. yt-dlp puede instalarse
    siguiendo las instrucciones de instalación del repositorio principal. Puedes instalar youtube-dl con el
    gestor de paquetes de Python, pip, que recomendamos para youtube-dl. Puedes hacerlo con el comando 
    <code>pip install youtube_dl</code>. Alternativamente, puedes instalarlo con el gestor de paquetes de tu sistema, 
    <code>apt install youtube-dl</code> en Ubuntu o <code>pacman -S youtube-dl</code> en Arch Linux.
</p>

## 🍰 Contribuciones:

Puedes contribuir al desarrollo de Plantita Ayudante siguiendo nuestro `todo.md` o abriendo un `issue` con alguna sugerencia para mejorar.

## 🛡️ Licencia:

Este proyecto tiene licencia Apache 2.0.

## Donaciones
Si has encontrado útil este proyecto y deseas apoyar su desarrollo continuo, considera hacer una donación. Tu contribución nos ayudará a:

- Mantener el proyecto actualizado con las últimas características y mejoras.
- Pagar por servicios y herramientas necesarias para el desarrollo y pruebas.
- Dedicar más tiempo y recursos a la documentación y soporte comunitario.

## ¿Cómo donar?
Puedes hacer una donación a través de cualquiera de las siguientes plataformas:
- [GitHub Sponsors](https://github.com/sponsors/CrawKatt)
- [Patreon](https://www.patreon.com/crawkatt)

Cualquier cantidad, grande o pequeña, es muy apreciada. ¡Gracias por tu apoyo!

¡Gracias por tu generosidad!

# Contribuidores
Muchas gracias a todos los que han contribuido a este proyecto. Son geniales.

<a href="https://github.com/crawkatt/plantita_ayudante/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=crawkatt/plantita_ayudante" />
</a>

Made with [contrib.rocks](https://contrib.rocks).

<p align="center"><img src="https://socialify.git.ci/crawkatt/plantita_ayudante/image?description=1&descriptionEditable=Bot%20de%20Discord%20del%20fan%20server%20de%20Meica&font=Source%20Code%20Pro&forks=1&issues=1&language=1&logo=https%3A%2F%2Fi.ibb.co%2FPZTwNYH%2F108593932-modified.png&name=1&owner=1&pattern=Floating%20Cogs&pulls=1&stargazers=1&theme=Dark" alt="project-image"></p>
