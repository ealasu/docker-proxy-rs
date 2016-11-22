#[macro_use] extern crate log;
extern crate env_logger;
extern crate iron;
extern crate utime;
extern crate hyper;
extern crate rustc_serialize;
extern crate openssl;

mod tee;
mod config;

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use iron::prelude::*;
use iron::status;
use iron::response::{BodyReader, WriteBody};
use iron::modifiers::Header;
use iron::headers::ContentLength;
use hyper::client::Client;
use hyper::status::StatusCode;


fn insecure_client() -> Client {
    use hyper::net::{HttpsConnector, OpensslClient};
    use openssl::ssl::*;

    //let mut ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    let mut ctx = SslContext::new(SslMethod::Sslv23).unwrap();
    ctx.set_options(SSL_OP_NO_SSLV2 | SSL_OP_NO_SSLV3 | SSL_OP_NO_COMPRESSION);
    // cipher list taken from curl:
    // https://github.com/curl/curl/blob/5bf5f6ebfcede78ef7c2b16daa41c4b7ba266087/lib/vtls/openssl.h#L120
    ctx.set_cipher_list("ALL!EXPORT!EXPORT40!EXPORT56!aNULL!LOW!RC4@STRENGTH").unwrap();
    ctx.set_verify(SSL_VERIFY_NONE, Some(verify));
    ctx.set_verify_depth(0);
    Client::with_connector(HttpsConnector::new(OpensslClient::new(ctx)))
}

fn verify(_: bool, _: &openssl::x509::X509StoreContext) -> bool {
    true
}

fn main() {
    env_logger::init().unwrap();

    let cache_dir = Path::new("/var/docker-cache");
    let config = config::read("/etc/docker-cache/config.json");

    let handler = move |req: &mut Request| {
        let cache_filename = cache_dir.join(
            req.url.path().join("__").replace(":", "__"));

        if cache_filename.is_file() {
            info!("hit");
            touch(&cache_filename);
            let meta = cache_filename.metadata().unwrap();
            Ok(Response::with((
                        status::Ok,
                        Header(ContentLength(meta.len())),
                        cache_filename)))
        } else {
            info!("miss");
            let url = format!("{}/{}", config.target, req.url.path().join("/"));
            info!("url: {}", url);
            let client = insecure_client();
            let res = client
                .get(&url)
                .headers(req.headers.clone())
                .send().unwrap();
            let res_len = match res.headers.get() {
                Some(&ContentLength(len)) => len,
                _ => 0
            };
            if res.status != StatusCode::Ok {
                return Ok(Response::with((
                            res.status,
                            Header(ContentLength(res_len)),
                            BodyReader(res))));
            }
            let body_writer: Box<WriteBody> = Box::new(tee::Tee::new(res, cache_filename));
            Ok(Response::with((
                        status::Ok,
                        Header(ContentLength(res_len)),
                        body_writer)))
        }
    };

    // openssl genrsa -out localhost.key 4096
    let key = Path::new("/etc/docker-proxy/app.key").to_path_buf();
    // openssl req -key localhost.key -x509 -new -days 3650 -out localhost.crt
    let cert = Path::new("/etc/docker-proxy/app.crt").to_path_buf();

    match Iron::new(handler).https("0.0.0.0:2222", cert, key) {
        Ok(listening) => println!("{:?}", listening),
        Err(err) => panic!("{:?}", err),
    };
}

fn touch<P: AsRef<Path>>(filename: P) {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    utime::set_file_times(filename, now, now).unwrap();
}
