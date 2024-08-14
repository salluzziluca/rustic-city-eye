# Agregado Final: Reconocimiento de Imágenes

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
2. [Proveedor de Infraestructura en la nube](#proveedor-de-infraestructura-en-la-nube)
    1. [Detalles y costos del proveedor](##detalles-y-costos-del-proveedor)
    2. [Uso del proveedor](##uso-del-proveedor)
3. [MultiThreading](#multithreading)
    1. [ThreadPool](##threadpool)
    2. [Sistema de Multithreading implementado](##sistema-de-multithreading-implementado)


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
