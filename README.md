Este es un proyecto para facilitar hacer rol (no necesariamente D&D) online, quiero que sea fácil de usar, código abierto, funcione a través de la red sin servidores.

La idea es una aplicación donde todo lo que hay en la pantalla es mera información visual de apoyo, con el objetivo de que no exista una mecánica de un rol en concreto sin implementar y que por culpa de eso no puedas actuar en consecuencia.

Existe un banco de zonas y mapas, de manera análoga es como si cada zona fuese una cajón donde dentro tienes guardado varios mapas y sacas el que quiera, pudiendo alternar, borrar, renombrar y reordenar cuando quieras. Arrastrando una imagen desde fuera del programa se crea el mapa de la zona activa, o si arrastras una imagen a un mapa ya creado se sustituye la imagen asiganda.

Los dados definido en interfaz son los típicos de 4, 6, 8, 10, 12 y 20, sin embargo esto no limita nada, a parte de que puedes aumentar el número de dados que tiras simultáneamente con la rueda del ratón o pulsando varias veces seguidas, restar lo que acabas de poner con click derecho entre otros, puedes simplemente escribir el comando equivalente en el chat donde salen las tiradas (de normal cuando pulsas un dado en la interfaz te sale el comando solo, es decir, el propio programa te enseña como funciona), y sin ningún tipo de limitación (excepto tirar más de 999 dados de golpe para evitar congelar el programa). Ejemplo, 123d46325. De manera adicional mientras el chat tenga el foco puedes pulsar flecha arriba o abajo para navegar entre el historial de tiradas.

Cuando seleccionas un mapa en concreto se pone en el tablero con una rejilla preconfigurada por el usuario (tiene valores por defecto en función de la resolución de la imagen adjunta a modo de predicción) que se puede configurar en cualquier momento.

Tiene un sistema de guardado mixto de imagen de la partida y por logs (concretamente por eventos, pero es más fácil entenderlo como logs), todos los cambios que hagas se van acumulando en un jsonl y cada 10 minutos se vuelca en la partida y se vacía. Este sistema permite que ante cierres inesperados, cortes de luz, u otra cosa, no se pierda ni una acción de las que haya hecho, ya que al volver a abrir el programa simplemente comprueba si quedan logs sin volcar.

La gestión básica de fichas ya está disponible: se crean desde el banco, sus datos e imagen se pueden editar después y se pueden marcar como únicas. Los jugadores aparecen en el banco en todas las zonas; los NPCs y enemigos se filtran por la zona actual. Las fichas se arrastran al mapa y se mueven por celdas de la rejilla.

Las fichas tienen dos estados, activas e inactivas. Una entidad genérica puede tener varias instancias, cada una con su propia vida; una entidad única solo puede estar colocada una vez a la vez y conserva sus datos al retirarla.

Las fichas son de tres tipos: jugador, NPC y enemigo. Las fichas activas se muestran resumidas y se pueden seleccionar para consultar y modificar sus parámetros. La visibilidad de vida y armadura se configura por separado. La sincronización de red y el intercambio de información privada entre participantes siguen siendo objetivos futuros.

El tablero, como he explicado anteriormente, tiene una rejilla, donde la idea que cada cuadrado representa un metro (medida internacional por favor, nada de gringolandia). En esta rejilla puedes arrastrar una ficha pre creada para instanciarla, permitiéndote arrastrarla por el mapa a donde quieras, pero encajándole en los cuadrados. Por defecto el "cuerpo" de una ficha en el tablero ocupa exactamente un cuadrado, sin embargo esto se puede cambiar a unidades no simétricas, como un 2x3, por si quieres representar un bicho más grande. La velocidad y el rango del arma descrito anteriormente se representan en una instancia cuando este coge el foco, en el momento que lo coges para arrastrarlo no se deja de dibujar de donde estaba con el objetivo de ver de manera cómoda hasta donde lo puedes arrastrar (no limito a que no puedas moverlo fuera por lo descrito anteriormente, imagínate que se teletransporta o algo, es fuera del rango del movimiento normal pero técnicamente puede hacerlo)

En la primera etapa del proyecto (la actual) la idea es que el rolmaster lo haga todo, tipo mueva fichas de los jugadores, tire los dados, etc. No es lo ideal y encima tendría que transmitir la pantalla por un tercero como discord para que pueda funcionar, complicando que al jugador no le llegue información específica.

La segunda etapa arregla lo anterior, ya que la idea es la implementación de funciones de red, donde reciba y envíe paquetes (intentaré que sea híbrido, con servidores gratuitos para la conexión inicial y peer to peer para el resto) y se sincronice toda la información en la pantalla. Esto permite que solo envíes lo que te interese garantizando la privatización de información.

En una tercera etapa muy lejana se pensarán cosas como "Dungeon creator", muros que tapen visibilidad, pero no tengo claro que termine de encargar con la visión del programa, ya se verá.

Para instalar las dependencias y compilar en Ubuntu 26.04, consulta la [guía de Ubuntu](./UBUNTU-26.04.md).
