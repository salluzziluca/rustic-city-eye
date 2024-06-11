# Taller de Programacion - Agentes Autonomos de Prevencion

## Grupo: Rustic City Eye
- Carranza, Lihuén.
- Demarchi, Ignacio.
- Giacobbe, Juan Ignacio.
- Salluzzi, Luca.

## Requisitos previos
- Rust y Cargo instalados en su sistema. Puede instalar Cargo y Rust desde [aqui](https://www.rust-lang.org/tools/install)

## Como levantar un Broker
1. Abra una terminal.
2. Ejecute el siguiente comando, reemplazando `[puerto]` con el numero de puerto deseado:

    ```sh
    cargo run --bin broker [puerto]
    ```

    Ejemplo:

    ```sh
    cargo run --bin broker 5000
    ```
## Como ejecutar la Aplicacion de Monitoreo
1. Una vez que se tiene un Broker corriendo, abra otra terminal.
2. En esta nueva terminal, ejecute el siguiente comando:

    ```sh
    cargo run --bin monitoring_app
    ```




## Como testear
- En una terminal, ejecute el siguiente comando:

    ```sh
    cargo test
    ```
