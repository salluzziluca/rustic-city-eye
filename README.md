# Taller de Programacion - Agentes Autonomos de Prevencion

## Grupo: Rustic City Eye
- Carranza, Lihuén.
- Demarchi, Ignacio.
- Giacobbe, Juan Ignacio.
- Salluzzi, Luca.

## Requisitos previos
- Rust y Cargo instalados en su sistema. Puede instalar Cargo y Rust desde [aqui](https://www.rust-lang.org/tools/install).
- Proyecto en Google Cloud Vision AI

## Cómo levantar un Broker
1. Abra una terminal.
2. Ejecute el siguiente comando, reemplazando `[puerto]` con el numero de puerto deseado:

```sh
cargo run -p broker [puerto]
```

Ejemplo:

```sh
cargo run -p broker 5000
```

### Cierre manual del Broker

- Al estar ejecutando el Broker, el usuario tiene la posibilidad de ingresar comandos en la misma terminal. 
- Un comando soportado por el Broker es el "shutdown", que lo que va a hacer es cerrar el Broker manualmente.

[Broker_example](https://github.com/user-attachments/assets/3c03b49d-5cac-4807-b492-8b693b0907b4)


```sh
shutdown
```

- Esto terminara con la ejecucion del Broker, cerrando todas las conexiones que esten corriendo.

## Cómo ejecutar la Aplicacion de Monitoreo
1. Una vez que se tiene un Broker corriendo, abra otra terminal.
2. En esta nueva terminal, ejecute el siguiente comando:

```sh
cargo run -p ui
```

[UI](https://github.com/user-attachments/assets/3d61e941-4d68-45ce-acfd-58b0df70a837)




## Cómo testear
- En una terminal, ejecute el siguiente comando:

```sh
cargo test -- --test-threads=1
```

## Agregado Final: Reconocimiento de Imágenes

Una vez que tengamos configurado nuestro proyecto en Gcloud para utilizar la tecnologia de Cloud Vision AI, procederemos a generar una nueva clave
publica, y tendremos que setearla en nuestro Sistema Operativo como una variable de entorno:

1. Editar el archivo de configuración del shell:

Dentro de nuestro directorio personal(~), ejecutamos lo siguiente:

- Usando Bash:

```sh
nano ~/.bashrc
```

- Usando Zsh:

```sh
nano ~/.zshrc
```

2. Agregar la variable de entorno:

Agregar al final del archivo abierto el siguiente comando:

```sh
export GOOGLE_API_KEY="tu_clave_publica"
```

3. Guardar y cerrar el archivo.

4. Recargar el archivo de configuración:

- Si se usó Bash:
```sh
source ~/.bashrc
```

- Si se usó Zsh:

```sh
source ~/.zshrc
```
