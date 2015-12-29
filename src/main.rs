use std::env;
use std::process;
use std::str;
use std::io::{Cursor, Error, ErrorKind, Read};

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
use xml::attribute::OwnedAttribute;
use xml::reader::{EventReader, XmlEvent};

type IoError = std::io::Error;
type HyperError = hyper::error::Error;
type XmlError = xml::reader::Error;

const SPEEDTEST_CONFIG:&'static str = "https://www.speedtest.net/speedtest-config.php";

#[derive(Debug)]
struct Config {
    client: Vec<OwnedAttribute>,
    times: Vec<OwnedAttribute>,
    download: Vec<OwnedAttribute>,
    upload: Vec<OwnedAttribute>
}

#[derive(Debug)]
enum SpeedtestError {
    Other(IoError),
    Http(HyperError),
    XmlParse(XmlError),
}

impl From<XmlError> for SpeedtestError {
    fn from(err: XmlError) -> SpeedtestError {
        SpeedtestError::XmlParse(err)
    }
}

impl From<HyperError> for SpeedtestError {
    fn from(err: HyperError) -> SpeedtestError {
        SpeedtestError::Http(err)
    }
}

impl From<IoError> for SpeedtestError {
    fn from(err: IoError) -> SpeedtestError {
        SpeedtestError::Other(err)
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    println!("{}", opts.usage(&brief));
}

struct XmlParser {
    body: String
}

impl XmlParser {
    fn new(data: String) -> XmlParser {
        XmlParser {
            body: data
        }
    }

    fn find_key(&self, key: &str) -> Result<XmlEvent, SpeedtestError> {
        let mut parser = EventReader::from_str(&*self.body);
        loop {
            let evnt = try!(parser.next());
            match evnt {
                XmlEvent::StartElement { ref name, .. } if name.local_name == key => { return Ok(evnt.clone()); },
                _ => { continue; }
            }
        }
    }

    fn find_key_attrs(&self, key: &str) -> Result<Vec<OwnedAttribute>, SpeedtestError> {
        match self.find_key(key) {
            Ok(XmlEvent::StartElement { name, attributes, .. }) => { return Ok(attributes); },
            Ok(_) => { return Err(SpeedtestError::from(Error::new(ErrorKind::Other, "Unknown Error!"))); },
            Err(e) => { return Err(e); }
        }
    }

}

fn get_config() -> Result<Config, SpeedtestError> {
    //Gather config data from speedtest
    let mut resp = try!(Client::new().request(Method::Get, SPEEDTEST_CONFIG).header(UserAgent("speedtest/0.0.1".to_owned())).send());
    info!("code={}; headers={};", resp.status, resp.headers);
    let mut body = String::new();
    resp.read_to_string(&mut body);

    let parser = XmlParser::new(body);

    Ok(Config {
        client: try!(parser.find_key_attrs("client")),
        times: try!(parser.find_key_attrs("times")),
        download: try!(parser.find_key_attrs("download")),
        upload: try!(parser.find_key_attrs("upload"))
    })
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

    let c = get_config();
    info!("Config is {:?}", c);
}
