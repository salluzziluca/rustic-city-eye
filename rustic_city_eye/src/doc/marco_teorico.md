# Marco Teórico

## Bibliografia
>
> <https://www.ibm.com/topics/message-queues>
>
> <https://www.ibm.com/topics/middleware>
>
> <https://blog.back4app.com/mqtt-vs-amqp/>
>
><https://mqtt.org/>

## MQTT(Message Queuing Telemetry Transport)
>
>publishers publish messages to a topic in a broker. The broker then broadcasts these messages to all the consumers subscribed to the topic.

==El mensaje llega, quizas 2 veces, quizas medio medio, pero llega==
![image](https://hackmd.io/_uploads/SkWf8CBe0.png)

### Pro

- Ya lo encontramos implementado
- It is simpler and more lightweight, making it easier to implement on resource-constrained devices and suitable for IoT applications. It uses a lightweight packet structure that reduces the data transmitted over the network.
- Provides great scalability and is more efficient than AMQP
- MQTT supports SSL/TLS for encryption and authentication. It also provides a simple username/password mechanism for client authentication.

### Cons

- It is more lightweight and sacrifices some advanced features for simplicity and efficiency. While it offers basic quality of service (QoS) levels for message delivery, it may not provide the same level of reliability as AMQP in all scenarios.

## AMQP(Advanced Message Queuing Protocol)
>
> Messages are published to an exchange in the broker first. The exchange acting as the routing agent, forwards these messages to the appropriate queue using its routing rules.

==El mensaje llega BIEN, PERFECTO, PETACULAR==

![image](https://hackmd.io/_uploads/Skrx8ASg0.png)

### Pro

- AMQP protocol supports a more sophisticated routing mechanism.
- The protocol defines not only the message format but also the rules for interactions between the entities in the messaging system. This includes how to establish a connection, maintain a session, and ensure secure communication.
- Soporta mas protocolos de comunicacion (ademas del publisher-subscriber).
- The protocol supports a wide range of messaging patterns and has strong delivery guarantees, including at-most-once, at-least-once, and exactly-once message delivery. It also provides flexible message routing features, which makes it a powerful tool for building complex distributed systems.
![message_delivering](https://hackmd.io/_uploads/S1HcHCSeA.png)
- In terms of reliability and message delivery, AMQP generally provides stronger guarantees than MQTT. This ensures that messages are reliably delivered to the intended recipients even in the face of network or system failures.
- AMQP, on the other hand, provides more robust security features. It supports SSL/TLS for encryption and SASL for authentication and integrity. AMQP also provides support for secure multi-tenancy, which allows multiple users to share the same messaging system while keeping their messages private and separate.
- MQTT, being lightweight, requires less computational power and hence less expensive hardware. It also uses less network bandwidth, which can save you money in terms of data usage costs.

### Contras

- It has a more complex packet structure that supports a wide range of messaging patterns.
- Requires more computational resources due to its complexity. This means you’ll need more powerful hardware.

## Libro MQTT Essentials

### Intro

- Lightwight and binary protocol, and MQTT excels when transferring data over the wire in comparison to protocols like HTTP.
- MQTT is a Client Server publish/subscribe messaging transport protocol. It is lightweight, open, simple, and designed so as to be easy to implement. These characteristics make it ideal for use in many situations, including constrained environments such as for communication in Machine to Machine (M2M) and Internet of Things (IoT) contexts where a small code footprint is required and/or network bandwidth is at a premium.”
- On October 29, 2014, MQTT became an officially approved OASIS Standard.

### PUB/SUB Pattern

- Decouples the client that sends a message (the publisher) from the client or clients that receive the messages (the subscribers). The publishers and subscribers never contact each other directly. In fact, they are not even aware that the other exists. The connection between them is handled by a third component (the broker).
- The job of the broker is to filter all incoming messages and distribute them correctly to subscribers.

![taller1](https://hackmd.io/_uploads/rk4EKc8x0.png)

#### Scalability

operations on the broker can be highly parallelized and messages can be processed in an event-driven way

![taller2](https://hackmd.io/_uploads/S1vvc5UgA.png)

#### WARNINGS

- both publisher and subscriber need to know which topics to use.
- The publisher can’t assume that somebody is listening to the messages that are sent. In some instances, it is possible that no subscriber reads a particular message.
- To keep the hierarchical topic tree flexible, it is important to design the topic tree very carefully and leave room for future use cases.

![taller3](https://hackmd.io/_uploads/S11FocLgA.png)

#### MQTT VS Message Queues

![taller4](https://hackmd.io/_uploads/HkESnqIeR.png)

### Client, Broker and Connection Establishment

#### Client

- Todos los clientes son publishers and subscribers.
- An MQTT client is any device that runs an MQTT library and connects to an MQTT broker over a network.
- Basically, any device that speaks MQTT over a TCP/IP stack can be called an MQTT client.

#### Broker

- El corazón de cualquier protocolo pub/sub.
- Is responsible for receiving all messages, filtering the messages, determining who is subscribed to each message, and sending the message to these subscribed clients.
- It also holds the sessions of all persisted clients, including subscriptions and missed messages.
- Authenticate and authorize the clients.
- It is important that your broker is highly scalable, integratable into backend systems, easy to monitor, and (of course) failure-resistant.

#### MQTT Connection

- Based on TCP/IP. Both the client and broker nedd to have a TCP/IP stack.
![taller5](https://hackmd.io/_uploads/rkIUA9LgC.png)

![taller6](https://hackmd.io/_uploads/SJTtC98eR.png)

#### CONNECT MESSAGE

The client sends a command message to the broker. If this CONNECT message is
malformed (according to the MQTT specification) or too much time passes between opening a network socket and sending the connect message, the broker closes the connection.

![taller7](https://hackmd.io/_uploads/SJxk6ysLlC.png)

![taller8](https://hackmd.io/_uploads/rkGebjLxC.png)

#### CONNACK MESSAGE

When a broker receives a CONNECT message, it is obligated to respond with a CONNACK message.

![taller9](https://hackmd.io/_uploads/SJBdZoLgA.png)

![taller10](https://hackmd.io/_uploads/rJ-eGiIgC.png)

### Publish, Subscribe and Unsubscribe

#### Publish

- An MQTT client can publish messages as soon as it connects to a broker
- Each message must contain a topic that the broker can use to forward the message to interested clients. Typically, each message has  a payload which contains the data to transmit in byte format.

![taller11](https://hackmd.io/_uploads/rk2r-gdlC.jpg)

- Topic Name: is a simple string that is hierarchically structured with forward slashes as delimiters.
- The service level determines what kind of guarantee a message has for reaching the intended recipient.
- PARA QoS levels, VER <https://www.hivemq.com/blog/mqtt-essentials-part-6-mqtt-quality-of-service-levels/>
- Retain Flag: This flag defines whether the message is saved by the broker as the last known good value for a specified topic.
- Payload: This is the actual content of the message.
- Packet Identifier: The packet identifier uniquely identifies a message as it flows between the client and broker.
- DUP Flag: The flag indicates that the message is a duplicate and was resent because the intended recipient (client or broker) did not acknowledge the original message.

![taller12](https://hackmd.io/_uploads/HyQLzlOe0.jpg)

#### Subscribe

![taller13](https://hackmd.io/_uploads/rkwaGeuxA.jpg)

![taller14](https://hackmd.io/_uploads/S1CfQlde0.jpg)

#### Suback

![taller15](https://hackmd.io/_uploads/S1IoXe_lR.jpg)

![taller16](https://hackmd.io/_uploads/H1FpXede0.jpg)

#### Unsubscribe, Unsuback

![taller17](https://hackmd.io/_uploads/rknmEgOlR.jpg)

### Topics and Best Practices

![taller18](https://hackmd.io/_uploads/rJffSedgC.jpg)

The client does not need to
create the desired topic before they publish or
subscribe to it. The broker accepts each valid
topic without any prior initialization.

![taller19](https://hackmd.io/_uploads/Bk3CPldxC.jpg)
