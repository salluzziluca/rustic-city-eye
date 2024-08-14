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

El servidor mantiene un registro de los clientes conectados y desconectados. En primer lugar, implementamos un Hash de clientes con sus streams correspondientes. En segundo lugar, implementamos un Hash de clientes con sus mensajes pendientes, de esta manera al volver a levantarlos, pueden recibirse facilmente los mensajes pendientes. 

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

---
# Agregado final - Reconocimiento de Imagenes

# Introducción

Para este agregados, tuvimos que incorporar la tecnologia de reconocimiento de imagenes al sistema central de camaras desarrollado en nuestro proyecto de Agentes 
Autonomos de Prevencion. Esta tecnologia incorpora una inteligencia artificial que nos permite interpretar y comprender el contenido de imagenes digitales, utilizando algoritmos de aprendizaje automatico como las redes neuronales convolucionales, para identificar y clasificar objetos, y tambien para identificar caracteristicas visuales de nuestras imagenes.

Las aplicaciones de seguridad y vigilancia suelen servirse de esta tecnologia, debido a que podemos detectar ciertos eventos u objetos a traves de nuestros dispositivos de seguridad, y tomar medidas al respecto. Para nuestra aplicacion, tuvimos que hacer capaz al sistema central de camaras de procesar imagenes para detectar potenciales incidentes en la via publica(por ejemplo incendios, accidentes de transito, etc). Luego de detectar un incidente en las imagenes, el sistema de camaras debe hacer uso del sistema de mensajeria para publicar un mensaje que describa este incidente y, en consecuencia, generar el incidente correspondiente y poner en marcha el circuito de resolucion de incidentes implementado en el proyecto. 


# Proveedor de Infraestructura en la nube

Para servirnos de un modelo y una infraestructura previamente preprocesados y entrenados, utilizamos el servicio de [Vision AI](https://cloud.google.com/vision?hl=es_419) que nos provee Google. Este proveedor nos ofrece una capa de uso gratuito mas que suficiente para probar el trabajo realizado en una demostracion final en vivo, debido a que podemos hacer uso de la misma con una carga de procesamiento mediana(minimo de 10 requests por minuto).

## Detalles y costos del proveedor

### Por que usamos Google Vision AI

1. Google Vision AI nos ofrece una capa gratuita ideal para fines de prueba y demostraciones en vivo: 1000 unidades gratuitas por mes para las API de visión. Esto es más que suficiente para nuestras necesidades, permitiendo hasta 1000 requests mensuales, lo cual supera el requerimiento mínimo de 10 requests por minuto requeridas para nuestra demostracion final en vivo.
2. Facilidad de Integración y Documentación Extensa: Cuenta con una documentación extensa y detallada, que facilita la integración con sistemas desarrollados en Rust. Esto es crucial para asegurar una integración rápida y eficiente con el sistema central de cámaras.
3. Variedad y Calidad de Servicios: Nos servimos especificamente de la feature de `label detection` y la feature que clasifica contenido explicito. Creemos que son las herramientas que necesitamos para detectar los distintos incidentes.
4. Ecosistema de Google Cloud: Google Vision AI permite beneficiarnos del ecosistema de Google Cloud, que ofrece una amplia variedad de herramientas y servicios complementarios. Esto puede facilitar la escalabilidad futura y la integración con otras tecnologías de Cloud.


### Analisis de Costos y performance

Supongamos que el sistema de cámaras realiza un promedio de 10 solicitudes por minuto en funcionamiento continuo. Esto equivale a:

- Solicitudes por hora: 10 solicitudes/minuto * 60 minutos/hora = 600 solicitudes/hora
- Solicitudes por día: 600 solicitudes/hora * 24 horas/día = 14,400 solicitudes/día
- Solicitudes por mes: 14,400 solicitudes/día * 30 días/mes = 432,000 solicitudes/mes

El servicio de Google tiene un costo promedio de $1.50 por cada 1000 imagenes a procesar. Por lo que haciendo ese numero de peticiones al mes, tendriamos un total de $646.50/mes.

El servicio es rapido a la hora de hacerle peticiones, en promedio se tarda de 0.35 a 0.6 segundos. Teniendo en cuenta el costo total, y el alto rendimiento que esta IA posee, creemos que es una gran opcion para la deteccion de incidentes en el contexto de nuestra aplicacion.

## Uso del proveedor

En un principio, el usuario debe tener creado un proyecto en Google Cloud, y debe habilitar el servicio de Vision AI. Una vez generado, debe crear una API key para realizar las peticiones de usuario. Nuestro programa funciona si el usuario tiene seteada una variable de entorno `GOOGLE_API_KEY` con la key de su proyecto.

Para hacer peticiones de una forma ordenada al proveedor, hemos optado por declarar al struct [Image Classifier][surveilling::annotation::ImageClassifier] que se encarga de tomar las imagenes y hacer la peticion correspondiente para etiquetar a la imagen.

Para hacer uso del clasificador de imagenes, se debe proveer un url sobre el cual vamos a hacer nuestras peticiones, ademas de un path al archivo que contenga las palabras claves para detectar incidentes. La incorporacion de este archivo de palabras claves nos parece una buena decision para permitirle al usuario la definicion de sus propio set de palabras claves para definir incidentes.

Antes de avanzar, cabe destacar que el servicio de Vision AI tomara imagenes, y nos devolvera etiquetas sobre la misma, con un score determinado(este score nos dira que tan confiable es la etiqueta, obviamente esto depende del preprocesamiento y entrenamiento que Google hizo sobre el modelo). Aqui hay un ejemplo tomado de la pagina oficial:

![dog_result](https://i.ibb.co/PQSymVH/dog-result.png)

En este caso, el modelo nos indica que la imagen contiene un perro(con un score de 0.96), y nos brinda mas caracteristicas sobre la misma. Para manejar mejor los resultados, la misma herramienta nos da la posibilidad de obtener la respuesta en formato JSON, lo cual nos parece mas acertado y comodo para trabajar con las peticiones, ya que vamos a hacer uso de los crates externos `serde` y `serde_json` para facilitar la serializacion y deserializacion de los documentos.

Para detectar incidentes, optamos por utilizar dos filtros que nos provee la API: `LABEL_DETECTION` y `SAFE_SEARCH_DETECTION`: la primera nos permite detectar etiquetas sobre la imagen, y la segunda nos permite detectar imagenes con contenido explicito(sirviendonos del detector que Google tiene integrado). Tambien, hemos modificado la cantidad maxima de resultados que nos da el modelo(por defecto son 10), y lo que hemos hecho fue setearlos con 50 resultados para label_detection, y con 10 para safe_search_detection.

El clasificador de imagenes que hemos declarado funciona de la siguiente manera para etiquetar las imagenes: se le provee un path hacia una imagen local, y se pasa a codificarla en base 64(haciendo uso del crate externo `Base64`), luego se realiza la request a la API, haciendo uso de un Client del crate externo `reqwest` en modo Blocking: esto nos permite manejar peticiones HTTP de manera sincronica, ya que va a bloquear el thread en ejecucion hasta que reciba una response. Las requests van a serializarse, y las responses van a deserializarse, obteniendo asi un vector de tuplas `(String, f64)`: el String corresponde a la etiqueta, y el f64 corresponde al score de esa etiqueta.

Al obtener el vector de etiquetas con sus respectivos scores, se pasa a detectar posibles incidentes, y es que si alguna de esas etiquetas contiene una palabra clave para detectar incidentes(puede ser por ejemplo la palabra `Fire`), se indica que un incidente fue detectado.  
![alt text](https://i.ibb.co/VjZPX2j/image.png)

# MultiThreading

## ThreadPool

Debido a la necesidad de emplear una estrategia de multithreading que nos permita ejecutar varias requests en paralelo, el equipo optó por implementar una ThreadPool. 
Una Threadpool es un mecanismo para manejar y ejecutar múltiples tareas de manera concurrente utilizando un grupo fijo de threads(el numero de threads es brindado por el usuario).

### Estructura de la Threadpool

1. ThreadPool

La estructura principal es ThreadPool, que contiene un vector de Workers y un sender de un canal mpsc.

    workers: Un vector que almacena los Workers que están disponibles para ejecutar tareas.
    sender: Permite enviar trabajos (Jobs) a los Workers.

2. Worker

Cada Worker es responsable de ejecutar los trabajos que recibe. Un Worker tiene un ID único y un thread que se ejecuta en un bucle esperando recibir trabajos.

    id: Identificador único del Worker.
    thread: El hilo en el que se ejecutan los jobs.

3. Job

Un Job es una tarea que se va a ejecutar en un thread. En nuestra implementación, se define como un alias de tipo (type) para un Box que contiene una función que toma la propiedad de sí misma (FnOnce), se puede enviar a través de threads (Send) y tiene una vida estática ('static).


### Uso de la ThreadPool

1. Creación de la ThreadPool: El método `new` de ThreadPool crea una nueva instancia del grupo de threads. Toma como parámetro el número de threads que se desean en el grupo.

2. Creación de un Worker: El método `new` de Worker toma un ID y un receptor de canal. Se crea un nuevo hilo que se ejecuta en un loop, esperando recibir jobs del canal. Cuando recibe un job, lo ejecuta.

3. Ejecución de tareas en la ThreadPool: El método `execute` en ThreadPool permite agregar nuevos jobs a la cola de tareas. Toma una función f que se ejecutará como una tarea en uno de los threads del grupo. También crea un canal interno para devolver el resultado de la ejecución.

## Sistema de Multithreading implementado

Se implementó un sistema mediante el cual, cuando se crea una nueva camara, se crea tambien un directorio asociada a esta. Con su ID como nombre del dir. Se desarrolló un Watcher que se encarga de monitorear un directorio en busca de nuevas imágenes. (el Watcher en cuestion tambien se utiliza para verificar la correcta creacion de directorios de camaras, las cuales se pueden verificar mediante el logging por consola).

El proceso se da dentro del metodo función run_client en el CameraSystem. Este metodo del CameraSystem maneja la recepción y el procesamiento de diferentes mensajes reenviados por el cliente. Dependiendo del tipo de mensaje, se activan o desactivan las cámaras cercanas a la ubicación del incidente. Tambien en el se maneja un Watcher que esta pendiente a cambios dentro de los directorios de las distintas camaras del sistema: Este es capaz de encontrar nuevos directorios(los cuales son creados apenas se crea una camara nueva en la aplicacion), y tambien es capaz de detectar nuevos archivos en el mismo(cuando el usuario ingresa las imagenes que quiere que sean procesadas por las camaras).

### Threads dentro del metodo client_run

1. El primer hilo se encarga de ejecutar el cliente del CameraSystem (camera_system_client.client_run()).
2. El segundo hilo se dedica a recibir y procesar mensajes de incidentes y resoluciones de incidentes. Inicialmente, se define un Receiver que puede ser un parámetro opcional o el receptor del cliente del CameraSystem.
3. El tercer hilo se encarga de vigilar el directorio de imágenes. Utiliza una `ThreadPool` para gestionar los jobs de procesamiento de cambios en el directorio de camaras.
Se crea un canal para recibir eventos de cambios en el directorio(los cuales son notificados por el Watcher). En un loop infinito, este thread espera recibir eventos de cambios en el directorio. Cuando se detecta un cambio, se decide si el evento debe ser procesado en función del tiempo transcurrido desde el último evento similar.
Si se debe procesar el evento, se llama a process_dir_change para manejar el cambio de directorio).

Dentro de este metodo, se spawnean los threads Dentro de run_client y se llama a watch_directory, un método del Watcher, para iniciar la vigilancia del directorio de imágenes.

El Watcher monitorea el directorio en busca de cambios. Cuando se detecta un cambio en el directorio, dentro de process_dir_change, CameraSystem maneja el cambio de directorio llamando a `analyze_image` y pasando el path de la imagen a clasificar.

La función `analyze_image` es una parte fundamental del `CameraSystem` para la detección de incidentes utilizando la tecnologia de procesamiento de imágenes. Utiliza un `ThreadPool` para procesar las imágenes de forma asíncrona, lo que permite que el sistema maneje múltiples imágenes simultáneamente sin bloquear el flujo principal del programa. 

`analize_image` primero obtiene la cámara que debe evaluar la imagen, luego esta se encarga de clasificar la imagen(haciendo uso de su `ImageClassifier`) y luego publica los incidentes detectados. Este proceso asegura que las imágenes sean evaluadas rápidamente y que cualquier incidente detectado sea reportado de inmediato.


El método `annotate_image` de la cámara, a su vez, llama a `annotate_image` en su `ImageClassifier` para procesar la imagen. Una vez que el `ImageClassifier` termina de procesar la imagen, devuelve un valor booleano indicando que la anotación se detecto un incidente(`true`), o no(`false`).

Finalmente, si el CameraSystem recibe un true, llama a `publish_incident` para publicar el incidente correspondiente.

![alt text](https://i.ibb.co/cNZ4hCx/classify-sequence1.png)


# Desgloce de la implementacion

El sistema está compuesto por tres módulos principales:

1. **Cliente de Mensajería (`run_client`)**
2. **Manejo de Mensajes de Incidentes (`handle_incident_messages`)**
3. **Supervisión de Directorios de Cámaras (`watch_dirs`)**

Cada uno de estos módulos interactúa para garantizar que el sistema de cámaras responda de manera efectiva a los incidentes reportados por el cliente.

## 3. Cliente de Mensajería

### 3.1. Función `run_client`

La función `run_client` es el punto de entrada principal del sistema, encargada de coordinar la ejecución del cliente de mensajería y los subsistemas que supervisan incidentes y directorios de cámaras. Recibe dos parámetros:

- `parameter_reciever`: Un receptor opcional de mensajes del cliente (`Option<Arc<Mutex<Receiver<ClientMessage>>>>`). Si no se proporciona, utiliza el receptor predeterminado del sistema de cámaras.
- `system`: Una referencia compartida y protegida al sistema de cámaras (`Arc<Mutex<CameraSystem<Client>>>`).

Esta función lanza tres hilos (`thread::spawn`):

1. **Hilo de Ejecución del Cliente**: Invoca el método `client_run` del sistema de cámaras para iniciar la conexión con el broker y recibir mensajes del cliente.
2. **Hilo de Manejo de Mensajes de Incidentes**: Ejecuta la función `handle_incident_messages` que procesa los mensajes relacionados con incidentes.
3. **Hilo de Supervisión de Directorios**: Inicia la función `watch_dirs` para monitorear cambios en los directorios de cámaras.

### 3.2. Método `client_run`

Dentro del hilo de ejecución del cliente, el método `client_run` del sistema de cámaras es llamado para establecer la conexión con el broker y comenzar a recibir mensajes. Si ocurre algún error durante la ejecución, este es capturado y registrado, pero el sistema sigue funcionando en otros aspectos.

## 4. Manejo de Mensajes de Incidentes

### 4.1. Función `handle_incident_messages`

La función `handle_incident_messages` es responsable de procesar los mensajes relacionados con incidentes y responder adecuadamente. Esta función se ejecuta en un bucle infinito y utiliza una copia del sistema de cámaras (`Arc::clone(&system)`) para evitar problemas de concurrencia.

Los pasos principales dentro del bucle son:

1. **Verificar y Procesar Incidentes Activos**: Si hay una ubicación de incidente pendiente, se intenta activar las cámaras cercanas utilizando el método `activate_cameras` del sistema de cámaras.
2. **Verificar y Procesar Incidentes Resueltos**: Si hay una ubicación de incidente resuelto pendiente, se intenta desactivar las cámaras cercanas utilizando el método `deactivate_cameras`.
3. **Recepción y Procesamiento de Mensajes**: Se bloquea el receptor para recibir un mensaje del cliente. Dependiendo del tipo de mensaje, se actualizan las ubicaciones de incidentes pendientes.

### 4.2. Función `process_client_message`

La función `process_client_message` es utilizada por `handle_incident_messages` para procesar los mensajes recibidos:

- Si el mensaje es un `publish` con el tópico `incident`, la ubicación del incidente es almacenada en `incident_location`.
- Si el mensaje es un `publish` con el tópico `incident_resolved`, la ubicación del incidente resuelto es almacenada en `solved_incident_location`.

Este procesamiento permite que las funciones `activate_cameras` y `deactivate_cameras` actúen en consecuencia.

## 5. Supervisión de Directorios de Cámaras

### 5.1. Función `watch_dirs`

La función `watch_dirs` supervisa el directorio de cámaras en busca de cambios (como la creación o modificación de archivos) y procesa estos eventos para garantizar que solo se manejen una vez, evitando duplicaciones debidas a posibles errores.

El proceso de supervisión se realiza en los siguientes pasos:

1. **Inicialización de un Pool de Hilos (`ThreadPool`)**: Un `ThreadPool` con 10 hilos se utiliza para manejar múltiples eventos simultáneamente.
2. **Observación de Directorios**: Utiliza la función `watch_directory` para monitorear un directorio específico.
3. **Procesamiento de Eventos**: En un bucle continuo, se reciben eventos a través de un canal (`channel`) y se determina si deben ser procesados, basándose en un control de tiempo (`last_event_times`). Si el evento no ha sido procesado recientemente, se invoca la función `process_dir_change`.

### 5.2. Función `process_dir_change`

La función `process_dir_change` se encarga de manejar los eventos de cambio detectados en los directorios:

- **Eventos de Creación de Archivos**: Si un archivo nuevo es detectado (especialmente imágenes con extensiones `.jpg`, `.jpeg`, `.png`), se invoca la función `analize_image` para que la cámara correspondiente analice la imagen.
- **Eventos de Creación de Directorios**: Si un nuevo directorio es detectado, se registra la creación y se notifica al sistema mediante logging.

### 5.3. Función `analize_image`

Cuando una imagen nueva es agregada al directorio de una cámara, la función `analize_image` localiza la cámara correspondiente en el sistema, y esta se encarga de analizar la imagen utilizando su método `annotate_image`.

- Si la imagen corresponde a un incidente, se invoca la función `publish_incident` para enviar un mensaje al broker con la ubicación del incidente.
- Si no es un incidente, el proceso se registra como tal y no se toma ninguna acción adicional.

## 6. Publicación de Incidentes

### 6.1. Función `publish_incident`

La función `publish_incident` es responsable de enviar un mensaje al broker cuando una cámara detecta un incidente. Este mensaje incluye la ubicación del incidente y se basa en la configuración de publicación (`PublishConfig`) que se lee de un archivo de configuración específico (`publish_incident_config.json`).

Los pasos clave en la publicación del incidente son:

1. **Obtención de la Ubicación de la Cámara**: Se extrae la ubicación desde la cámara que detectó el incidente.
2. **Creación de la Carga Útil (`IncidentPayload`)**: Se genera una carga útil con la información del incidente.
3. **Envío del Mensaje**: Utilizando el método `send_message` del sistema de cámaras, se envía el mensaje al broker.

En caso de errores durante este proceso, se manejan y registran adecuadamente para garantizar la estabilidad del sistema.
