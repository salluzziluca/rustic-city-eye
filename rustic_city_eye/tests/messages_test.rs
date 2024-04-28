use rustic_city_eye::mqtt::client::server_run;
use std::net::TcpStream;

#[test]
fn test_connect() {
    let addrs = "127.0.0.1:5000";
    Broker::server_run(addrs).unwrap();
    let mut stream = TcpStream::connect(addrs).unwrap();
    let connect = ClientMessage::Connect {};
    connect.write_to(&mut stream).unwrap();
}
