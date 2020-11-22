mod socks5;

use std::net::{TcpStream, Shutdown, TcpListener};
use crate::socks5::socks5::Socks5;

fn main(){
    let lis = TcpListener::bind("0.0.0.0:1080").unwrap();
    for stream in lis.incoming() {
        std::thread::spawn(move||{
            handle_stream(stream.unwrap());
        });
    }

}
fn handle_stream(stream:TcpStream){
    let mut request = Socks5::new(stream);
    request.serve();
    println!("over!");
}