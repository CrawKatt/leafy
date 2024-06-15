# Tareas Pendientes

---

- [ ] Refactorizar el código para que sea más legible
- [x] Normalizar la Base de Datos para que sea más eficiente y evitar la repetición de datos
  - [x] Crear una sola tabla de configuración
  - [x] Si no hay una tabla de configuración al establecer un valor,
  crearla con el valor establecido y el resto de los valores por defecto
- [ ] Enviar un sticker de "que dijiste @muteado"
- [x] Crear un sistema de reacciones para borrar un ooc si se reacciona con un emoji 10 veces

---

# Tareas Completadas

---
- [x] Almacenar los mensajes en la Base de Datos para logearlos
- [x] Advertir al usuario que mencionó a un usuario o un rol prohibido a través del Bot
- [x] Obtener los mensajes enviados
- [x] Obtener los mensajes editados desde el `edited_message_handler`
- [x] Obtener los mensajes borrados desde la Base de Datos
- [x] Crear un comando para setear el canal de Logs
- [x] Crear un comando para obtener el canal de Logs establecido
- [x] Crear una función que establezca `TimeOut` al usuario que mencione a un usuario prohibido
- [x] Crear un comando para establecer el ID del rol de `TimeOut`
- [x] Crear un sistema de logs para los mensajes editados
- [x] Crear un comando para establecer el ID del usuario prohibido (prohibido de mencionar)
- [x] Crear un comando para establecer el ID del rol prohíbido (prohíbido de mencionar)
- [x] Crear una función que establezca `TimeOut` al usuario que mencione a un usuario perteneciente al rol prohibido
- [x] Crear una excepción para que el Bot no borre los mensajes de los usuarios con el rol de `Administrador` o `Moderador` cuando se menciona al usuario o al rol prohibido
- [x] Crear un comando para establecer el rol de `Administrador` para que el Bot haga excepciones con los usuarios que tengan ese rol
- [x] Crear un comando para obtener el rol de `Administrador` establecido
- [x] Crear un comando para establecer el tiempo de `TimeOut`
- [x] Borrar el mensaje donde se menciona a un usuario o un rol prohibido a través del Bot
- [x] Borrar el mensaje donde se menciona a un usuario o un rol prohíbido a través del Bot si el mensaje fue editado y se menciona a un usuario o un rol prohibido
- [x] Logear el mensaje borrado por el Bot cuando se menciona a un usuario o un rol prohibido en un mensaje
- [x] Proporcionar en el autocompletado de `set_timeout_timer` el tiempo de `TimeOut` establecido
- [x] Mover todos los `println!();` de error a `log_error!();`
- [x] Cambiar la función de excepción para que el Bot busque a todos los roles administradores establecidos
- [x] Cambiar el mensaje de `time_out` de Hardcode a dinámico mediante un comando que almacene el mensaje personalizado en la Base de Datos
- [x] Cambiar los mensajes de Advertencia de Hardcode a dinámicos mediante un comando que almacene el mensaje personalizado en la Base de Datos
- [x] Cambiar el tiempo de `TimeOut` de hardcode a una tiempo establecido desde la Base de Datos
- [x] Añadir una excepción para que el Bot no tome en cuenta si el usuario prohíbido se menciona a si mismo
- [x] Corregir el error de que el Bot detecta a cualquier usuario como un usuario prohíbido si se menciona en un mensaje
- [x] Enviar mensaje de sugerencia pidiendo responder un mensaje sin mencionar a un usuario o un rol prohibido
- [x] Restringir el uso de comandos de administración a los usuarios con el rol de `Administrador`
- [x] Añadir un comando que desactive y reactive la función de usuario prohíbido si este está de acuerdo (el comando solo debe poder usarlo el usuario prohíbido o un administrador)
- [x] Crear un comando que active y desactive la función de broma del Bot
- [x] Realizar una broma a Meica como presentación del Bot cuando interactúe con el la primera vez. La broma debe desactivarse automáticamente antes de terminar la función
- [x] Cambiar los campos de structs por tipos primitivos y crear métodos para obtener los valores de los campos
- [x] Crear un comando para establecer un canal exclusivo en donde se enviará la broma del Bot
- [x] Arreglar el embed para enviar los audios al canal de Logs
- [x] Corregir Bug en el comando get_admins que no muestra todos los roles de administrador
- [x] Corregir Bug en el comando set_admins que reptie el primer rol de administrador cuando se establecen dos roles
- [x] Cambiar método de sanción de rol a `timeout`
- [x] Crear un sistema anti-spam para impedir el envío de links maliciosos
- [x] Cambiar los mensajes de advertencia por embeds
- [x] Crear tests unitarios para las funciones del Bot
- [x] Crear una librería con FFMPEG para convertir formatos de audio
- [x] Mejorar el sistema anti spam
- [x] Remover los embeds para casos de mención de usuario o rol prohibido (Los embeds no mencionan aunque usen @ en el mensaje)
- [x] Añadir modo auth al docker-compose (CRÍTICO)
- [x] Crear un comando screenshot con embeds para fuera de contexo
- [x] Crear un comando para obtener el top de palabras usadas por un usuario
- [x] Crear un mensaje de bienvenida para los miembros nuevos en el canal de Bienvenida
  - [x] Crear un sistema de edición de imágen para añadir el avatar del usuario nuevo en una plantilla de bienvenida
  - [x] Crear un comando para establecer el canal de Bienvenida
  - [x] Crear un comando para obtener el canal de Bienvenida establecido
  - [x] Crear un comando para establecer el mensaje de Bienvenida
---