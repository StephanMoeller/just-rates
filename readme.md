This project is my first real attempt do create something useful in Rust. (C# developer as profession)

Overall project:

A server instance, that allows publishers to send information about processings from instance A to instance B at a certain rate.
Cunsumers are then able to view these metrics in realtime and create realtime network graphs, showing an entire systems interactions with highlighting of the areas that are currently at work.

Details:

Producers send usind UDP. Consumers consume using an HTTP api with consequtive polling.

Producer Protocol:

- Works over UDP
- Strings are sent as utf-8 bytes. They are expressed as "PROTOCOL MESSAGE HERE" where the "" are not part of the strings to be sent.
- Client can send the string: "!SHOULD I SEND" to the server (without the "") and the server will reply with "!PLEASE SEND" or "!PLEASE SLEEP". Client should send this message type every 5 seconds to check up on whether it must send or not.
- Server can send "!PLEASE SEND" to client to inform the client to start/continue sending metrics to the server. The server may initiate this message itself or as a response to a "!SHOULD I SEND" message from the client.
- Server can send "!PLEASE SLEEP" to client to inform the client to stop sending metrics to the server. The server may initiate this message itself or as a response to a "!SHOULD I SEND" message from the client.
- Client can send metrics data to the server in the format "#FROM TO 23" where "FROM" is the origin, "TO" is the receiver and "23" is the processed count since the previous metrics message. NOTE: "FROM" and "TO" cannot contain white spaces as that would conflict with the white space used as delimiters.
- In case a message if wrongfully formatted or an error occurred while processing it, the server will respond to the client with the message "ERROR Some error message here". Note that the error message can contain white spaces.

Running tests:
Tests should be run with nextest using this commando: cargo nextest run
(Install on windows using this powershell script listed here: https://nexte.st/book/pre-built-binaries.html)