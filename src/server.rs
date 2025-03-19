use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use std::sync::{Arc};
use std::collections::HashMap;
use tokio::sync::Mutex;

/** Main function, it will call and await on the run_server function, and, in the 
* case of any error in running the server, it will print the error.
*/
#[tokio::main]
async fn main() {
  if let Err(e) = run_server().await {
    eprintln!("Error running server: {}", e);
  }
}

/**
* Function to be called by main. This is where connections to ther server are 
* validated and assigned their own thread. Uses tokio::spawn to create asynchronous threads.
*/
async fn run_server() -> Result<(), Box<dyn std::error::Error>>{
  // establish Connections variable, which maintains a hashmap of client_id to OwnedWriteHalf for all currently connected clients
  let connections = Arc::new(Mutex::new(Connections::new()));
  // establish a TcpListener which is bound to localhost port 8888
  let listener = tokio::net::TcpListener::bind("127.0.0.1:8888").await.unwrap();
  println!("listening on port 8888");
  // loop to continually accept/connect to clients connecting to 127.0.0.1:8888
  loop {
    // wait until a socket connects to port
    let (stream, socket_addr) = listener.accept().await.unwrap();
    // assign client id to be the port of the socket address
    let c_id = socket_addr.port();
    // clone connections via Arc so you are free to pass it in as an argument to handle_connection
    let connections = Arc::clone(&connections);
    println!("connected {} {}", socket_addr.ip(), c_id);
    // spawn a thread to asynchronously manage this socket/connection until
    tokio::spawn(async move {
      handle_connection(stream, connections, c_id).await;
  });
  }
}

/** 
* Function which reads in messages from each client. When a message is recieved
* an acknowledgement is sent to the client which sent the message, and the message 
* (as well as the client_id of the sender) is sent to all currently connected clients.
*/
async fn handle_connection(stream: tokio::net::TcpStream, connections: Arc<Mutex<Connections>>, c_id: u16){
  //split TcpStream into OwnedReadHalf and OwnedWriteHalf
  let (read, write) = stream.into_split();
  //acquire lock for connections, and insert mapping of client ID to OwnedWriteHalf for this client
  connections.lock().await.cons.insert(c_id, write);
  //acquire lock for connections and send this client login acknowledgement
  connections.lock().await.login_msg(c_id).await;
  // create reader from which to read in messages (as lines) from the client
  let reader = tokio::io::BufReader::new(read);
  let mut read = reader.lines();
  // loop to continually read next line/message sent from this client
  loop  {
    // wait until next line/message sent by client, and store message in this variable
    let line_result = read.next_line().await;
    match line_result {
      //Got a new line/message
        Ok(line) => {
          //if (and only if) the message is not None, broadcast it to all other clients and send sender an acknowledgement
          if let Some(line) = line {
            println!("message {} {}", c_id, line);
            let mut connections = connections.lock().await;
            let msg = format!("MESSAGE:{} {}\n", c_id, line);
            //broadcast function handles both broadcasting message to all others and sending acknowldegemnt to sender
            connections.broadcast(c_id, msg).await;
          }
        },
        //In case of an error (e.g. client disconnected) remover client from map of connections
        Err(_) =>{
          //acquire lock on connections
          let mut connections = connections.lock().await;
          //remove current client from connections
          connections.cons.remove(&c_id);
          //break from loop to stop awating messages from disconnected client
          break;
        }
    }
  }
}

/**Struct to maintain mapping of client IDs to corresponding OwnedWriteHalf to 
* enable message broadcasting. Struct is used to manage ownership via Arc and Mutex.
*/
struct Connections{
  cons : HashMap<u16, OwnedWriteHalf>,
}

/** Implementation for Connections struct. 
* Implements constructor function, fn new().
* Implements function used to broadcast specified messages from specified clients.
* Implements function to send login acknowledgement to specified client.
*/
impl Connections {
  // Constructor. Create a new, empty, instance of `Connections`.
  fn new() -> Self {
    Connections {
          cons: HashMap::new(),
      }
  }
 
  /** Function which takes in c_id (client ID of the sender) and a string message.
  * It iterates through all currently connected clients (i.e. those with mappings contained in self.cons)
  * and writes through the corresponding OwnedWriteHalf either a message acknowledgement (to the sender)
  * or the message (to all other clients).
  */
  async fn broadcast(&mut self, c_id: u16, message: String){
    // for each connection/mapping in the map for connections
    for con in self.cons.iter_mut() {
      // if the client id of the mapping is not the client id of the message sender
      if *con.0 != c_id {
        // Attempt to write message to this connection via corresponding OwnedWriteHalf, print error if any occur
        if let Err(e) = con.1.write(message.as_bytes()).await {
          eprintln!("Failed to send data to client_id {}: {}", con.0, e);
        }
      }
      // Otherwise, the client id matches and this is the client that sent the message
      else {
        // Attempt to write message acknowledgement via corresponding OwnedWriteHalf, print error if any occur
        if let Err(e) = con.1.write(b"ACK:MESSAGE\n").await {
          eprintln!("Failed to send data to client_id {}: {}", c_id, e);
        }
      }
    }
  }

  /** Function which takes in c_id (client ID of the sender).
  * This function will attempt to write a login acknowledgement to clients that just connected.
  * If there is an error in sending the data, it is printed to the server terminal.
  */
  async fn login_msg(&mut self, c_id: u16,){
    // Format login message
    let login_msg = format!("LOGIN:{}\n", c_id);
    //Attempt to write login Acknowlegement, print error if any occur
    if let Err(e) = self.cons.get_mut(&c_id).unwrap().write(login_msg.as_bytes()).await {
      eprintln!("Failed to send data to client_id {}: {}", c_id, e);
    }
  }
}