//! Se conecta mediante TCP a la dirección asignada por argv.
//! Lee lineas desde stdin y las manda mediante el socket.

use std::env::args;
use std::io::stdin;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;

use rustic_city_eye::mqtt::broker_message::BrokerMessage;
use rustic_city_eye::mqtt::client_message::ClientMessage;

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

    let mut socket = TcpStream::connect(address)?;
    
    let connect = ClientMessage::Connect { client_id: 1 };
    println!("Sending connect message to broker: {:?}", connect);
    connect.write_to(&mut socket).unwrap();

    let connack = BrokerMessage::read_from(&mut socket);
    println!("recibi un {:?}", connack);

    for line in reader.lines() {
        if let Ok(line) = line {
            println!("Enviando: {:?}", line);
            socket.write_all(line.as_bytes())?;
            socket.write_all("\n".as_bytes())?;
            {
                let mut server_response = BufReader::new(&mut socket);
                let mut response = String::new();
                server_response.read_line(&mut response)?;
                println!("Respuesta del sv: {:?}", response);
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
