# RustBroadcastServer
This project is a TCP broadcast server built with Rust.

# Running the Server:
To run the server you can run the following command from the root of the project

```
cargo run
```

The server connects to local host (127.0.0.1) port 8888.

The server will print to terminal in the following cases:

- Successfully connects to port 8888 (```listening on port 8888```)
- When a client connects (```connectted {ip_address} {client_id}```)
- When a message from a client is recieved (```message {client_id} {message_txt}```) 

# Connecting Clients
For a client to connect, first ensure the server is currently running on localhost, then simply run ```nc localhost 8888```.

When a client connects, a client_id is assigned to the client, matching the port from which they connect to port 8888.

Upon connecting, clients will recieve a login acknowledgement (ex. ```LOGIN:{CLIENT_ID}``).

After sending a message, client will recieve a message acknowledgement (ex. ```ACK:MESSAGE```).

After the server recieves a message from one client, all other clients will then recieve the message (ex. ```MESSAGE:{CLIENT_ID} {MESSAGE}```), where CLIENT_ID is the ID of the sender.

# Implementation Details

There are some important notes to be made about the server implementation.
- Currently, the server always connects to localhost 8888. If needed, it is possible to make modifications that would
allow for the server to connect to other ports (for example, via commandline arguments).
- Currently, the clinet ID generated for a client is simple the port of the socket address, which is retrieved upon the client connecting.
If there is need to improved client ID assignment, other methods could be employed to ensure each client has a unique ID.
- This is a multi-thread approach which utilizes tokio, futures (via async/await).
- High Level Design: main calls a functions which connects the server to the port and listens for new clients. When clients connect
to the server, this function spawns an asynchronous thread which calls a function dedicated to listening for messages from that individual client. 

