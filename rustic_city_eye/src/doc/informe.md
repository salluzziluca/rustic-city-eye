# Proyecto: Agentes Autónomos de Prevención

**Materia**: Taller de Programación.

Primer Cuatrimestre de 2024.

## Grupo: Rustic City Eye

### Integrantes

- Carranza, Lihuén.
- Demarchi, Ignacio.
- Giacobbe, Juan Ignacio.
- Saluzzi, Luca.

# Índice

1. [Introducción](#introducción)
    1. [¿Qué es un Sistema de Mensajería Asincrónico?](##qué-es-un-sistema-de-mensajería-asincrónico)
    2. [¿Qué es un Message Broker?](##qué-es-un-message-broker)
    3. [Protocolos de Mensajería Asincrónica](#protocolos-de-mensajería-asincrónica)
        1. [Advanced Message Queuing Protocol(AMQP)](#advanced-message-queuing-protocolamqp)
        2. [Message Queuing Telemetry Transport(MQTT)](#message-queuing-telemetry-transportmqtt)
    4. [¿Qué protocolo vamos a implementar?](#que-protocolo-vamos-a-implementar)
2. [Servicio de Mensajería](#servicio-de-mensajería)
3. [Aplicación de Monitoreo](#aplicación-de-monitoreo)
4. [Sistema Central de Cámaras](#sistema-central-de-cámaras)
5. [Software de Control de Agentes(Drones)](#software-de-control-de-agentesdrones)

# Introducción

## ¿Qué es un Sistema de Mensajería Asincrónico?
>
>Un **Sistema de Mensajería Asincrónico** es una plataforma que permite la comunicación entre diferentes partes sin que estas necesiten estar simultáneamente activas. Ambos participantes de una conversación tienen la libertad de comenzar, pausar y resumir conversaciones en sus propios términos, sin necesidad de tenerlos conectados a tiempo real.

Al utilizar este tipo de sistemas de mensajería, distintos procesos se comunican entre sí intercambiando mensajes de una manera asincrónica. Un cliente introduce un comando o una petición a un servicio a través de un mensaje, y si este necesita responder, envía un mensaje otra vez al mismo cliente.

Debido a que estamos en un contexto de comunicación asincrónica, el cliente asume que la respuesta no va a ser recibida instantáneamente, y que incluso puede no tenerla. No necesitamos que los procesos que se comunican estén intercambiando mensajes a tiempo real.

Aplicar este tipo de mensajería evita la pérdida de datos valiosos y permite que los sistemas sigan funcionando incluso ante los problemas intermitentes de conectividad o latencia habituales en las redes públicas. La mensajería asíncrona garantiza que los mensajes se entreguen una vez (y sólo una vez) en el orden correcto con respecto a otros mensajes.

Un ejemplo cotidiano en el que usamos este tipo de sistemas de mensajerías es al intercambiar correos electrónicos con otra persona: nosotros enviamos un mensaje, y el receptor del mismo no tiene la necesidad de respondernos al instante(incluso, puede que no obtengamos respuesta...). No tenemos la necesidad de tener a ambas partes conectadas a tiempo real.

## ¿Qué es un Message Broker?

> Un **Message Broker** es un software que permite la comunicación entre distintas aplicaciones, sistemas y servicios que forman parte de un sistema de comunicación. Podemos pensarlo como el intermediario entre las partes que se van a comunicar.

Un Message Broker tiene como principal función traducir los mensajes entre protocolos de mensajería formales. Esto permite que servicios interdependientes "hablen" directamente entre sí, aunque estén escritos en lenguajes diferentes o se implementen en plataformas distintas.

![broker](https://i.ibb.co/ZNR5BDd/image.png)

Los Message Brokers son capaces de validar, almacenar, encaminar y entregar mensajes a los destinos apropiados. Sirven de intermediarios entre aplicaciones, permitiendo a los remitentes emitir mensajes sin saber dónde están los receptores, si están activos o no, o cuántos hay. Esto facilita el desacoplamiento de procesos y servicios dentro de los sistemas.

Para proporcionar un almacenamiento de mensajes fiable y una entrega garantizada, los Message Brokers suelen basarse en una subestructura o componente denominado cola de mensajes que almacena y ordena los mensajes hasta que las aplicaciones consumidoras pueden procesarlos. En una cola de mensajes, los mensajes se almacenan en el orden exacto en que fueron transmitidos y permanecen en la cola hasta que se confirma su recepción.

Los Message Brokers pueden incluir gestores de colas para manejar las interacciones entre múltiples colas de mensajes, así como servicios que proporcionan funciones de enrutamiento de datos, traducción de mensajes, persistencia y gestión del estado del cliente.

## Protocolos de Mensajería Asincrónica
>
> Los **Protocolos de Mensajería Asincrónica** son conjuntos de reglas y estándares que facilitan la entrega eficiente de mensajes entre sistemas o aplicaciones sin la necesidad de una respuesta inmediata.

### Advanced Message Queuing Protocol(AMQP)

En este protocolo, los mensajes se publican primero en una bolsa del Broker. La bolsa, actuando como agente de enrutamiento, reenvía estos mensajes a la cola adecuada utilizando sus reglas de enrutamiento.

![amqp](https://i.ibb.co/JxY9SLg/image.png)

#### Ventajas de utilizar el protocolo

- Soporta un mecanismo de enrutamiento sotisficado.
- No solo define el formato del mensaje, sino también define las reglas de interacción entre las entidades del sistema de mensajería. Esto incluye cómo establecer una conexión, cómo mantener una sesión y cómo garantizar una comunicación segura.
- Admite una amplia gama de patrones de mensajería y ofrece sólidas garantías de entrega, como la entrega de mensajes at-most-once, at-least-once, and exactly-once. También ofrece funciones flexibles de enrutamiento de mensajes, lo que lo convierte en una potente herramienta para construir sistemas distribuidos complejos.
- En términos de fiabilidad y entrega de mensajes, AMQP asegura que los mensajes se entreguen de forma fiable a los destinatarios previstos, incluso ante fallos de la red o del sistema.
- Ofrece funciones de seguridad sólidas. Soporta SSL/TLS para cifrado y SASL para autenticación e integridad. AMQP también ofrece soporte para multi-tenancy seguro, que permite a varios usuarios compartir el mismo sistema de mensajería manteniendo sus mensajes privados y separados.

#### Desventajas de utilizar el protocolo

- La implementación y configuración inicial de AMQP puede ser más compleja en comparación con otros protocolos más simples. Requiere un entendimiento más profundo de los conceptos de colas, exchanges y enrutamiento, lo cual puede implicar una curva de aprendizaje mayor para los desarrolladores y administradores de sistemas.
- Aunque AMQP está optimizado para el rendimiento, el uso de características avanzadas como confirmaciones de entrega y colas persistentes puede requerir más recursos computacionales y de almacenamiento en comparación con protocolos más ligeros y menos robustos.
- Debido a su flexibilidad y configurabilidad, la implementación inicial y el mantenimiento continuo de una infraestructura basada en AMQP pueden requerir más tiempo y recursos en comparación con soluciones más simples y menos personalizables.

AMQP es una opción sólida para construir una red de mensajería asincrónica en el contexto de nuestro proyecto gracias a su robustez, escalabilidad y garantías de entrega de mensajes.

### Message Queuing Telemetry Transport(MQTT)

En este protocolo, los publishers publican mensajes en un topic de un Broker. A continuación, el mismo Broker difunde estos mensajes a todos los consumidores suscritos al topic.

![mqtt](https://i.ibb.co/C7pjQgC/image.png)

#### Ventajas de utilizar el protocolo

- Es más sencillo y ligero, lo que facilita su uso en dispositivos con recursos limitados. Utiliza una estructura de paquetes ligera que reduce los datos transmitidos por la red.
- Admite SSL/TLS para cifrado y autenticación. También proporciona un sencillo mecanismo de nombre de usuario/contraseña para la autenticación del cliente.
- Al ser ligero, requiere menos potencia de cálculo y, por tanto, un hardware menos costoso. También utiliza menos ancho de banda de red, lo que puede ahorrarle recursos a las aplicaciones y/o sistemas que lo usen.
- Utiliza un modelo de publisher/subscriber (pub/sub) que facilita la comunicación asincrónica entre múltiples dispositivos y aplicaciones.
- Está diseñado para manejar conexiones inestables y pérdidas ocasionales de conectividad de red, lo cual es común en entornos móviles o con cobertura variable. En el contexto del proyecto, los drones pueden mantener la conexión y reanudar la transmisión de datos de vigilancia cuando la conectividad se restablezca.

#### Desventajas de utilizar el protocolo

- Aunque MQTT es adecuado para un gran número de dispositivos conectados, puede enfrentar limitaciones en términos de escalabilidad horizontal en comparación con protocolos como AMQP, que están optimizados para manejar volúmenes muy grandes de mensajes y conexiones.
- Al estar diseñado para ser simple y eficiente, carece de algunas características avanzadas que son ofrecidas por otros protocolos más complejos como AMQP. Por ejemplo, MQTT no soporta enrutamiento avanzado de mensajes o colas persistentes de la misma manera que AMQP.
- Aunque MQTT ofrece opciones de seguridad básicas como autenticación y cifrado TLS/SSL, puede no ser tan robusto como AMQP en términos de características de seguridad avanzadas y personalización.
- A diferencia de AMQP, que puede manejar desconexiones temporales y reintentos automáticos de entrega de mensajes, MQTT depende en gran medida de la disponibilidad continua de la red para la entrega de mensajes en tiempo real.

MQTT es una excelente opción para implementaciones de redes de mensajería asincrónica en entornos como la vigilancia con drones autónomos, debido a su simplicidad, eficiencia y soporte para pub/sub.

## ¿Que protocolo vamos a implementar?

Luego de haber estudiado distintos protocolos de message broking, como por ejemplo AMQP y MQTT, como así también los requerimientos del proyecto, **hemos decidido implementar el protocolo MQTT**.

Nuestras aplicaciones necesitan de un sistema de mensajería que sea eficiente y ligero, debido a la simpleza de los mensajes que estas van a enviarse entre sí, por lo que no necesitamos del formato estricto de los mensajes que provee AMQP.

Los mensajes que envían la Aplicación de Monitoreo, el Sistema Central de Cámaras y los Drones no necesitan de un formato muy estricto, debido a que se enviarán alertas con localizaciones(para el caso de la Aplicación de Monitoreo, que agregará incidentes, y el Sistema de Cámaras los detecta y le envía la alerta junto a la localización a los Drones), y actualizaciones de estados(en el caso del Drone por ejemplo deja de estar circulando en su área de operación para ir a su central a cargar su batería).

A diferencia de MQTT, AMQP requiere de más recursos computacionales y de almacenamiento, debido a su uso de estructuras más complejas.

Aunque AMQP por su parte es un protocolo más seguro y robusto que MQTT, consideramos que este último se adapta mejor a las necesidades de nuestro proyecto, por lo que decidimos por implementar este protocolo.

# Servicio de Mensajería

Luego de haber tomado la decisión de implementar el protocolo de message broking MQTT para nuestro proyecto, comenzamos con los requerimientos funcionales de nuestro servicio de mensajería.

Desde la cátedra se nos recomendó seguir el patrón de comunicación *publisher-suscriber*, y la arquitectura *cliente-servidor* para construir el servicio de mensajería.

## Patrón publisher-suscriber
>
> MQTT es un protocolo de mensajería específicos que sigue la arquitectura publisher-suscriber.  Utiliza un modelo basado en intermediarios en el que los clientes se conectan a un intermediario(en nuestro caso será el Broker) y los mensajes se publican en topics. Los suscriptores pueden suscribirse a temas específicos y recibir los mensajes publicados.

![pubsub](https://i.ibb.co/KKsjNVk/image.png)

### Desacoplamiento del Publisher y Suscriber

MQTT desacopla espacialmente al publisher y al suscriber, lo que significa que sólo necesitan conocer el nombre de host/IP y el puerto del broker para publicar o recibir mensajes. Además, MQTT desacopla por tiempo, lo que permite al Broker almacenar mensajes para clientes que no están en línea. Para almacenar mensajes deben cumplirse dos condiciones: que el cliente se haya conectado con una sesión persistente y se haya suscrito a un tema con una calidad de servicio superior a 0.

### Filtrado de Mensajes

MQTT utiliza el filtrado de mensajes basado en asuntos. Cada mensaje contiene un topic que el broker puede utilizar para determinar si un cliente suscriptor recibe el mensaje o no. Para manejar los desafíos de un sistema pub/sub, MQTT tiene tres niveles de Quality of Services (QoS). Se puede especificar fácilmente que un mensaje se entregue correctamente desde el cliente al broker o desde el broker a un cliente. Sin embargo, existe la posibilidad de que nadie se suscriba al topic en cuestión. Para mantener la flexibilidad del árbol de topics jerárquico, es importante diseñar el árbol de topics con mucho cuidado y dejar espacio para futuros casos de uso.

### Escalabilidad

Como MQTT sigue la arquitectura pub/sub, la escalabilidad es algo natural en este protocolo, lo que lo hace ideal para las aplicaciones que vamos a desarrollar. A pesar de sus ventajas, escalar a millones de conexiones puede suponer un reto para Pub/Sub. En estos casos, se pueden utilizar nodos Broker agrupados para distribuir la carga entre varios servidores, mientras que los balanceadores de carga pueden garantizar que el tráfico se distribuya uniformemente.

## Arquitectura Cliente-Servidor

> La arquitectura cliente-servidor es un modelo fundamental en la organización de sistemas de software y redes, donde las aplicaciones se dividen en dos partes principales: el cliente, que solicita y consume servicios, y el servidor, que provee recursos o servicios a través de una red. Esta estructura facilita la distribución de tareas y responsabilidades, permitiendo la escalabilidad, la modularidad y la centralización del control. En este modelo, los clientes envían peticiones al servidor, que responde proporcionando los datos solicitados o ejecutando acciones específicas, garantizando así una interacción eficiente y controlada entre los usuarios y los sistemas informáticos.

![clientserver](https://i.ibb.co/DbbMRRC/image.png)

Para MQTT, tenemos dos entidades principales: el Client, y el Broker. El Client va a enviar mensajes al Broker, éste los procesará, y enviará un llamado acknoledgement message al Client, indicando el estado del mensaje que se envió en un principio.

### Características del Client

- Todos los clientes son publishers y suscribers.
- Un cliente MQTT es cualquier dispositivo que ejecuta una librería MQTT y se conecta a un broker MQTT a través de una red.
- Básicamente, cualquier dispositivo que habla MQTT sobre un stack TCP/IP puede ser llamado un cliente MQTT.

### Características del Broker

- Es el corazón del protocolo.
- Es responsable de recibir todos los mensajes, filtrarlos, determinar quién está suscrito a cada mensaje y enviar el mensaje a estos clientes suscritos.
- También mantiene las sesiones de todos los clientes persistentes, incluidas las suscripciones y los mensajes perdidos.
- Es responsable de autenticar y autorizar a los clientes.

## Requerimientos del servicio

Tuvimos en cuenta ciertos requerimientos al desarrollar nuestro servicio de mensajería especificados en la consigna, y que fueron de ayuda para tomar la decisión final de implementar el protocolo MQTT.

### Seguridad

Se implementó un método de autenticación "password-based" el cual, como su nombre lo indica, es basado en contraseñas.

Al iniciar un Broker, éste levanta un archivo que contiene información sobre los distintos usuarios que están registrados, guardando su client_id, su username, y su password.

Cada vez que un Client quiera autenticarse, debe brindar en su primer paquete enviado al Broker(para MQTT siempre va a tratarse de un Connect) tanto su username como su contraseña. A partir de estos datos, el Broker verifica si este usuario existe, y verifica su contraseña. Si la verificación es exitosa, el Broker devuelve un Connack con reason code 0x00(conexión exitosa), y el comienza la conexión entre el Broker y el Client, donde van a comunicarse mediante el intercambio de los paquetes ya explicados.

### Calidad de servicio(QoS)

> Es un acuerdo entre el emisor de un mensaje y el receptor de un mensaje que define la garantía de entrega de un mensaje específico. La QoS ofrece al cliente la posibilidad de elegir un nivel de servicio que se ajuste a la fiabilidad de su red y a la lógica de su aplicación. Dado que MQTT gestiona la retransmisión de mensajes y garantiza la entrega (incluso cuando el transporte subyacente no es fiable), la QoS facilita enormemente la comunicación en redes poco fiables.

El protocolo MQTT en su versión 5.0(el que implementamos para el proyecto), soporta los niveles de 'at-most-once' y 'at-least-once':

- Para el nivel 'at-most-once', se garantiza una entrega 'best-effort'. No hay garantía de que el mensaje de haya entregado. El emisor envía el mensaje al Broker, y este ni siquiera envía un acknoledgement al emisor. El mensaje puede llegar o no.
- Para el nivel 'at-least-once', se garantiza que el mensaje se entrega por lo menos una vez al receptor. Además, el Broker devuelve un mensaje del tipo Puback al emisor, de forma tal de que este último sepa que su mensaje se ha entregado. Para este nivel es importante destacar que si el mensaje no se entrega a los receptores en un tiempo determinado, el Broker va a empezar a reenviarlo tantas veces sean necesarias hasta que llegue correctamente a los receptores.

### Reliability

El servidor mantiene un registro de los clientes conectados y desconectados. De esta manera, cuando los clientes pierden o eligen cortar su conexión, pueden volver a levantarla sin eprder las suscripciones anteriores.

### Configuración

Para el caso del broker, su puerto se indica en el comando de ejecución `cargo run --bin broker 5000. En el caso del cliente, se indica a través de la UI en los campos correspondientes.

### Logging

# Aplicación de Monitoreo

Es la aplicación principal del proyecto. La idea es que pueda recibir la carga de incidentes por parte del usuario, y notificar a la red ante la aparición de un incidente nuevo y los cambios de estado del mismo (iniciado, resuelto, etc).

La aplicación de monitoreo cuenta con una instancia de [Client][mqtt::client::Client], el cual se va a intentar conectar a un Broker corriendo en un puerto especificado desde la interfaz gráfica, y también se va a tomar un username y una password provenientes de la interfaz, procediendo así con la autenticación del Client. En caso de error, la aplicación no se cierra, sino que arroja una alerta del error, y se le pide al usuario que complete el formulario de "login" nuevamente.

La aplicación crea una instancia de un [Sistema Central de Cámaras][surveilling::camera_system::CameraSystem], y la idea es que junto a los agentes que el usuario agregue se comuniquen entre sí para resolver los incidentes que el usuario cree desde la interfaz gráfica.

En la misma se puede visualizar el estado completo del sistema, incluyendo el mapa geográfico de la región con los incidentes activos, y la posición y el estado de cada agente (dron) y cada cámara de vigilancia.

Esta posee 4 botones. Uno para agregar cámaras, otro para incidentes y 2 pertenecientes al sistema de Drones.

Para agregar un elemento al sistema uno debe primero seleccionar un punto en el mapa y luego elegir que tipo de entidad quiere crear. En el caso de los drones, es necesario primero crear una central de drones que los aloje, no se pueden crear drones sin una central.

Al crear/reportar un incidente, se activaran las camaras que esten en el rango requerido. El camera system recibirá un publish message mediante el broker que indicará que hubo un incidente en cierta Location. Este entonces activará las camaras que se encuentren a rango del incidente, asi como las camaras lindantes a las recientement activadas.

# Sistema Central de Cámaras

El [sistema de camaras ][surveilling::camera_system::CameraSystem]  es la entidad encargada de gestionar todas las camaras de la aplicación. Este tiene como tipo de dato principal un hashmap del tipo `<ID, Camera>`. A este se le puede pedir crear una camara nueva, o modificar al estado de las camaras actuales.
Cuando el sistema reciba del broker un incidente, levantar las camaras necesarias y enviará por medio de un publish todas las camaras *que hayan cambiado de estado*. De esta forma es el camera system el que se encarga de la logica del analisis de sus camaras y el resto de clientes solo reciben el diferencial de camaras modificadas. Este comportamiento se puede ver en [esta funcion.][surveilling::camera_system::CameraSystem::activate_cameras]

Cuando un incidente es resuelto, se desactivan las camaras utilizando un mecanismo practicamente identico al anterior([deactivate cameras][surveilling::camera_system::CameraSystem::deactivate_cameras]). Primero se desactivan las camaras que se encuentran en rango, y estas luego envian una señal a sus camaras vecinas para que tambien se apaguen, si asi corresponde.

# Software de Control de Agentes(Drones)

Para la gestion de agentes autonomos se utilizan 3 estructuras [drone system][drones::drone_system::DroneSystem]  , [drone_centers][drones::drone_center::DroneCenter] y [drones][dronee::Drone]. El drone system lleva apunte de sus drone centers y estos a su vez de cada uno de sus drones.

Cada drone es un cliente y recibe mediante publish las notifiaciones de los incidentes. En caso de que ir a resolver el incidente no le suponga alejarse de su rango maximo de operacion (determinado por su drone center), se dirigira hacia esa direccion.

Mientras no reciba ningun tipo de notificacion de accidente, el drone se quedará esperando, volando dentro de su rango de operacion. Cuando su bateria se agote, ira a cargarse a su drone center. Esto se logra mediante dos threads, uno que se encarga del movimiento del drone y otro de la carga y descarga de bateria

El dron tiene 3 estados: Waiting (esperando a recibir algun incidente), Attending Incident y Low Battery. Cuando se encuentre en este ultimo es cuand se dirigirá a su estación de carga.

Estos comportamientos se llevan a cabo en [run_drone][drones::drone::Drone::run_drone], con la utilizacion de [battery_discharge][drones::drone::Drone::battery_discharge] y [charge_battery][drones::drone::Drone::charge_battery] para la carga y descarga de bateria respectivamente.
