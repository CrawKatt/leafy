# Tareas Pendientes

---

- (**POST PRODUCCIÓN**)
- [ ] Refactorizar el código para que sea más legible
- [ ] Añadir un comando que desactive la función de usuario prohíbido si este está de acuerdo (el comando solo debe poder usarlo el usuario prohíbido)
- [ ] Restringir el uso de comandos de administración a los usuarios con el rol de `Administrador`

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
---