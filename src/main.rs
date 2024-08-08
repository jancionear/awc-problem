//! Expected flow:
//! 1. Create a new system
//! 2. Create a client - use custom connector and TLS so that disconnect_timeout is Some, otherwise ConnectionPool destructor won't do anything
//! 3. Complete a request
//! 4. The connection is returned to the ConnectionPool
//! 5. Make the client live longer by spawning a future that captures it
//! 6. The system is stopped
//! 7. Client is dropped and ConnectionPool destructor spawns a task to close the pooled connection. This panics because the system is already stopped.

use std::time::Duration;

use awc::{Client, Connector};

fn main() {
    let sys = actix::System::new();
    sys.block_on(async {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .connector(Connector::new().max_http_version(awc::http::Version::HTTP_11))
            .finish();
        let req = client.get("https://telemetry.nearone.org/nodes").send();
        match req.await {
            Ok(_) => {
                println!("OK!");
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        actix::spawn(async move {
            let c: Client = client;
            let _g = c.get("abcd");
            println!("Client has size: {}", std::mem::size_of_val(&c));
            actix::clock::sleep(Duration::from_secs(100000)).await;
        });
        actix::clock::sleep(Duration::from_secs(1)).await;
        actix::System::current().stop();
    });
    sys.run().unwrap();
    println!("Dropped system");

    std::thread::sleep(Duration::from_secs(1));
}
