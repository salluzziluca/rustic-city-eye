# Informe Trabajo Práctico 1C2024

## Grupo: Rustic City Eye

### Integrantes

Carranza, Lihuén
Demarchi, Ignacio
Giacobbe, Juan Ignacio
Saluzzi, Luca

## ¿Qué es un Sistema de Mensajería Asincrónico?

Un Sistema de Mensajería Asincrónico es una plataforma que permite la comunicación entre diferentes partes sin que estas necesiten estar simultáneamente activas.

Al utilizar este sistema de mensajería, distintos procesos se comunican entre sí intercambiando asincrónicamente mensajes. Un cliente introduce un comando o una petición a un servicio a través de un mensaje. Si el servicio necesita responder, envía un mensaje otra vez al cliente.

Debido a que estamos en un contexto de comunicación basada en mensajes, el cliente asume que la respuesta no va a ser recibida instantáneamente, y que incluso puede no tener respuesta. Este tipo de comunicación es asincrónico, debido a que no necesitamos que los procesos que se comunican estén intercambiando mensajes a tiempo real.

Aplicar este tipo de mensajería evita la pérdida de datos valiosos y permite que los sistemas sigan funcionando incluso ante los problemas intermitentes de conectividad o latencia habituales en las redes públicas. La mensajería asíncrona garantiza que los mensajes se entreguen una vez (y sólo una vez) en el orden correcto con respecto a otros mensajes.

Un ejemplo cotidiano en el que usamos este tipo de sistemas de mensajerías es al intercambiar correos electrónicos con otra persona: nosotros enviamos un mensaje, y el receptor del mismo no tiene la necesidad de respondernos al instante(incluso, puede que no obtengamos respuesta...). No tenemos la necesidad de tener a ambas partes conectadas a tiempo real.

> En definitiva, definimos a un **Sistema de Mensajería Asincrónico** como un sistema en el cual ambos participantes de una conversación tienen la libertad de comenzar, pausar y resumir conversaciones en sus propios términos, eliminando la necesidad de tener a amabas partes conectadas a tiempo real.

### Comunicación Asíncrona basada en Eventos

Al utilizar este sistema, un microservicio publica un evento de integración cuando ocurre algo dentro de su dominio y otro microservicio necesita tener conocimiento de ello, como un cambio de precio en un microservicio de catálogo de productos. Otros microservicios se suscriben a los eventos para poder recibirlos de forma asíncrona.

Cuando esto ocurre, los receptores pueden actualizar sus propias entidades de dominio, lo que puede provocar que se publiquen más eventos de integración. Este sistema de publicación/suscripción se realiza utilizando una implementación de un bus de eventos.

El bus de eventos puede diseñarse como una abstracción o interfaz, con la API necesaria para suscribirse o desuscribirse a eventos y para publicar eventos. El bus de eventos también puede tener una o más implementaciones basadas en cualquier broker inter-procesos y de mensajería, como una cola de mensajería o un bus de servicios que soporte comunicación asíncrona y un modelo de publicación/suscripción.

## ¿Qué es un Message Broker?

Un Message Broker es un software que permite la comunicación entre las distintas aplicaciones, sistemas y servicios que formen parte de nuestro sistema de comunicación. Podemos pensarlo como el intermediario entre las partes que se van a comunicar.

Esto lo logra traduciendo los mensajes entre protocolos de mensajería formales. Esto permite que servicios interdependientes "hablen" directamente entre sí, aunque estén escritos en lenguajes diferentes o se implementen en plataformas distintas.

![taller11](https://hackmd.io/_uploads/Hy86tcqxA.png)

> Los Message Brokers pueden validar, almacenar, encaminar y entregar mensajes a los destinos apropiados. Sirven de intermediarios entre otras aplicaciones, permitiendo a los remitentes emitir mensajes sin saber dónde están los receptores, si están activos o no, o cuántos hay. Esto facilita el desacoplamiento de procesos y servicios dentro de los sistemas.

Para proporcionar un almacenamiento de mensajes fiable y una entrega garantizada, los corredores de mensajes suelen basarse en una subestructura o componente denominado cola de mensajes que almacena y ordena los mensajes hasta que las aplicaciones consumidoras pueden procesarlos. En una cola de mensajes, los mensajes se almacenan en el orden exacto en que fueron transmitidos y permanecen en la cola hasta que se confirma su recepción.

Los Message Brokers pueden incluir gestores de colas para manejar las interacciones entre múltiples colas de mensajes, así como servicios que proporcionan funciones de enrutamiento de datos, traducción de mensajes, persistencia y gestión del estado del cliente.
