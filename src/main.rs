//! Expected flow:
//! 1. Start listening on a TCP port
//! 2. Create a new system, and spawn a future which creates a client and does a request
//! 3. The request hangs because the server doesn't respond
//! 4. The system is stopped
//! 5. Client is dropped and the destructor tries to spawn a future but it panics (doesn't happen :/)

use std::io::Read;
use std::time::Duration;

use awc::Client;

// Listen on a TCP port, accept one connection and read from it, but don't respond
fn listen() {
    let listener = std::net::TcpListener::bind("127.0.0.1:8320").unwrap();
    println!("Started listening");
    for stream_res in listener.incoming() {
        if let Ok(mut stream) = stream_res {
            let mut buf = [0; 1024];
            while let Ok(r) = stream.read(&mut buf) {
                if r > 0 {
                    println!("Read {} bytes", r);
                }
            }
            std::thread::sleep(Duration::from_secs(1000000));
        }
    }
}

fn run_sys() {
    let sys = actix::System::new();
    sys.block_on(async {
        actix::spawn(async move {
            let client = Client::builder().timeout(Duration::from_secs(5)).finish();
            let req = client.get("http://127.0.0.1:8320").send();
            match req.await {
                Ok(_) => {
                    println!("OK!");
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        });
        actix::clock::sleep(Duration::from_secs(1)).await;
        actix::System::current().stop();
    });
    sys.run().unwrap();
    println!("Dropped system");
}

fn main() {
    std::thread::spawn(listen);
    std::thread::sleep(Duration::from_secs(1));

    run_sys();

    std::thread::sleep(Duration::from_secs(10));
}
