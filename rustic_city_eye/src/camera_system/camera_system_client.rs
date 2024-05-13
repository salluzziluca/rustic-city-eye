//! Se conecta mediante TCP a la dirección asignada por argv.
//! Lee lineas desde stdin y las manda mediante el socket.

use std::env::args;
use std::io::{stdin, BufRead};

use std::io::{BufReader, Read};

use rustic_city_eye::mqtt::client::Client;

static CLIENT_ARGS: usize = 3;

fn main() -> Result<(), ()> {
    let argv = args().collect::<Vec<String>>();
    if argv.len() != CLIENT_ARGS {
        println!("Cantidad de argumentos inválido");
        let app_name = &argv[0];
        println!("{:?} <host> <puerto>", app_name);
        return Err(());
    }

    let address = argv[1].clone() + ":" + &argv[2];
    println!("Soy el Camera System, conectándome a {:?}", address);

    client_run(&address, &mut stdin()).unwrap();
    Ok(())
}

/// Client run recibe una dirección y cualquier cosa "legible"
/// Esto nos da la libertad de pasarle stdin, un archivo, incluso otro socket
fn client_run(address: &str, stream: &mut dyn Read) -> std::io::Result<()> {
    let reader = BufReader::new(stream);

    let mut client = match Client::new(address) {
        Ok(client) => client,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error al crear cliente",
            ))
        } //todo
    };

    for line in reader.lines() {
        if let Ok(line) = line {
            if line.starts_with("publish:") {
                let (_, post_colon) = line.split_at(8); // "publish:" is 8 characters
                let message = post_colon.trim(); // remove leading/trailing whitespace
                println!("Publishing message: {}", message);
                client.publish_message(message);
            } else if line.starts_with("subscribe:") {
                let (_, post_colon) = line.split_at(10); // "subscribe:" is 10 characters
                let topic = post_colon.trim(); // remove leading/trailing whitespace
                println!("Subscribing to topic: {}", topic);
                client.subscribe(topic);
            } else {
                println!("Comando no reconocido: {}", line);
            }
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error al leer linea",
            ));
        }
    }

    Ok(())
}
