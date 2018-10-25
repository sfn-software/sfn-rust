use std::io;
use std::result::Result;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;


const SFN_FILE: u8 = 0x01;
const SFN_DONE: u8 = 0x02;
const SFN_FILE_WITH_MD5: u8 = 0x03;


fn read_byte(mut stream: impl Read) -> io::Result<u8> {
	let mut buf: [u8; 1] = [ 0x00 ];
	stream.read_exact(&mut buf)?;
	Ok(buf[0])
}

fn send_files(mut stream: impl Write) -> io::Result<()> {
	println!("Local done.");
	stream.write(&[ SFN_DONE ])?;
	Ok(())
}

fn recv_files(mut stream: impl Read) -> io::Result<()> {
	loop {
		let opcode = read_byte(&mut stream)?;
		match opcode {
			SFN_DONE => {
				println!("Remote done.");
				return Ok(());
			},
			SFN_FILE => panic!("Unsupported SM opcode: {}", opcode),
			SFN_FILE_WITH_MD5 => panic!("Unsupported SM opcode: {}", opcode),
			_ => panic!("Unsupported SM opcode: {}", opcode),
		}
	}
}

pub fn handle_client(stream: TcpStream) -> io::Result<()> {
	let stream_clone = stream.try_clone().unwrap();
	let send_thread = thread::spawn(move || {
		send_files(&stream_clone).unwrap();
	});
	let recv_thread = thread::spawn(move || {
		recv_files(&stream).unwrap();
	});

	send_thread.join().unwrap();
	recv_thread.join().unwrap();
	println!("All done, closing connection.");
	Ok(())
}
