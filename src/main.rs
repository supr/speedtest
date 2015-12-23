use std::env;
use std::process;
use std::str;
use std::io::Read;
use std::collections::HashMap;

extern crate hyper;
use hyper::client::Client;
use hyper::header::UserAgent;
use hyper::method::Method;

extern crate getopts;
use getopts::Options;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate xml;
use xml::reader::EventReader;

const SPEEDTEST_CONFIG:&'static str = "https://www.speedtest.net/speedtest-config.php";

struct Config {
    client: HashMap<String, String>,
    times: HashMap<String, String>,
    download: HashMap<String, String>,
    upload: HashMap<String, String>
}

impl Config {
    fn new() -> Self {
        Config{
            client: HashMap::new(),
            times: HashMap::new(),
            download: HashMap::new(),
            upload: HashMap::new()
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size).map(|_| INDENT)
        .fold(String::with_capacity(size*INDENT.len()), |r, s| r + s)
}

fn get_config() -> Config {
    //Gather config data from speedtest
    let resp = Client::new().request(Method::Get, SPEEDTEST_CONFIG).header(UserAgent("Mozilla/5.0".to_owned())).send().unwrap();
    info!("code={}; headers={};", resp.status, resp.headers);

    //Begin XML Parse
    let parser = EventReader::new(resp);
    Config::new()
}

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help");
    opts.optflag("l", "list", "Display a list of speedtest.net servers sorted by distance");
    opts.optopt("s", "server", "Specify a server ID to test against", "SERVER");
    opts.optopt("t", "timeout", "HTTP timeout in seconds. Default 10", "TIMEOUT");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => { error!("{}", e.to_string());  process::exit(1); }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        process::exit(0);
    }

    let timeout: u16 = matches.opt_str("t").unwrap_or("10".to_string()).parse::<u16>().unwrap_or(10u16);
    let server_id: u16 = matches.opt_str("s").unwrap_or("0".to_string()).parse::<u16>().unwrap_or(0u16);

    info!("Timeout is {}", timeout);
    info!("Server ID is {}", server_id);

    get_config();
}
