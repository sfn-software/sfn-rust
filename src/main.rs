mod protocol;

use std::net::{TcpListener, TcpStream};
use std::io;

extern crate clap;
use clap::{Arg, App};


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
		.arg(Arg::with_name("files")
			.value_name("FILE")
			.help("Files to send")
			.takes_value(true)
			.multiple(true)
		)
		.get_matches();
	println!("{:?}", cli);

	let mut filenames: Vec<String> = Vec::with_capacity(25);
	if cli.is_present("files") {
		for x in cli.values_of("files").unwrap() {
			filenames.push(String::from(x));
		}
	}

	let listen_addr = cli.value_of("listen").unwrap_or("");
	let connect_addr = cli.value_of("connect").unwrap_or("");

	if listen_addr != "" && connect_addr != "" {
		panic!("Must use one of `--connect` or `--listen`, not both");
	}
	if listen_addr == "" && connect_addr == "" {
		panic!("Must use one of `--connect` or `--listen`");
	}

	if listen_addr != "" {
		return work_as_server(listen_addr, filenames);
	} else {
		return work_as_client(connect_addr, filenames);
	}
}


fn work_as_server(addr: &str, files: Vec<String>) -> io::Result<()> {
	let listener = TcpListener::bind(addr)?;
	println!("Listening at {}", addr);

	for stream in listener.incoming() {
		let stream = stream?;
		println!("Accepted connection from {}", stream.peer_addr()?);
		return protocol::handle_client(stream, files);
	}

	Ok(())
}


fn work_as_client(addr: &str, files: Vec<String>) -> io::Result<()> {
	println!("Connecting to {}", addr);
	let sock = TcpStream::connect(addr)?;
	println!("Connected");

	return protocol::handle_client(sock, files);
}
