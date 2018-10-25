extern crate clap;
use clap::{Arg, App};

use std::net::{TcpListener, TcpStream};
use std::io;

mod protocol;
use protocol::handle_client;


fn main() -> io::Result<()> {
	let cli = App::new("sfn-rust")
		.version("1.0.0")
		.author("m1kc (Max Musatov) <m1kc@yandex.ru>")
		.about("A simple program to send files over network, compatible with other sfn implementations.\nDefault port is 3214.")
		.arg(Arg::with_name("listen")
			.short("l")
			.long("listen")
			.value_name("IP:PORT")
			.help("Wait for connections")
			.takes_value(true)
		)
		.arg(Arg::with_name("connect")
			.short("c")
			.long("connect")
			.value_name("IP:PORT")
			.help("Connect to other host")
			.takes_value(true)
		)
		.get_matches();
	println!("{:?}", cli);

	let listen_addr = cli.value_of("listen").unwrap_or("");
	let connect_addr = cli.value_of("connect").unwrap_or("");

	if listen_addr != "" && connect_addr != "" {
		panic!("Must use one of `--connect` or `--listen`, not both");
	}
	if listen_addr == "" && connect_addr == "" {
		panic!("Must use one of `--connect` or `--listen`");
	}

	if listen_addr != "" {
		return work_as_server(listen_addr);
	} else {
		return work_as_client(connect_addr);
	}
}


fn work_as_server(addr: &str) -> io::Result<()> {
	let listener = TcpListener::bind(addr)?;
	println!("Listening at {}", addr);

	for stream in listener.incoming() {
		let stream = stream?;
		println!("Accepted connection from {}", stream.peer_addr()?);
		return handle_client(stream);
	}

	Ok(())
}


fn work_as_client(addr: &str) -> io::Result<()> {
	println!("Connecting to {}", addr);
	let sock = TcpStream::connect(addr)?;
	println!("Connected");

	return handle_client(sock);
}
