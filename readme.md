This project is my first real attempt do create something useful in Rust. (C# developer as profession)

Overall project:

A server instance, that allows publishers to send information about processings from instance A to instance B at a certain rate.
Cunsumers are then able to view these metrics in realtime and create realtime network graphs, showing an entire systems interactions with highlighting of the areas that are currently at work.

Details:

Producers send usind UDP. Consumers consume using an HTTP api with consequtive polling.

Producer Protocol:

- Works over UDP
- Strings are sent as utf-8 bytes. They are expressed as "PROTOCOL MESSAGE HERE" where the "" are not part of the strings to be sent.
- #1 PING: Client can send the string: "PING" to the server (without the "") and the server will reply with "PONG".
- #2 PONG: Server will reply a PING message with the message "PONG"
- #3 DATA: Client can send metrics data to the server in the format "DATA FROM TO 23" where "FROM" is the origin, "TO" is the receiver and "23" is the processed count since the previous metrics message, eg. "DATA ProductDb OrderWorker 7". NOTE: "FROM" and "TO" cannot contain white spaces as that would conflict with the white space used as delimiters.
- #4: ERROR: In case a message if wrongfully formatted or an error occurred while processing it, the server will respond to the client with the message "ERROR Some error message here". Note that the error message can contain white spaces.

Running tests:
Tests should be run with nextest using this commando: cargo nextest run
(Install on windows using this powershell script listed here: https://nexte.st/book/pre-built-binaries.html)