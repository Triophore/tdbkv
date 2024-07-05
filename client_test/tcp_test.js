const net = require("net");
const msgpack = require("@msgpack/msgpack"); // Install this library: npm install @msgpack/msgpack

const HOST = "127.0.0.1";  // Adjust to match your Rust server's address
const PORT = 1234; // Adjust to match your Rust server's port

const client = new net.Socket();

client.connect(PORT, HOST, function() {
    console.log("Connected to server!");

    const dataToSend = { 
        client_name: "Alice"  
    };

    // Encode data into MessagePack
    const encodedData = msgpack.encode(dataToSend);

    // Ensure the encoded data is sent as a buffer
    const dataBuffer = Buffer.from(encodedData);

    // Send the encoded data with a newline
    console.log(dataBuffer)
    
    client.write(JSON.stringify(dataToSend));
    console.log("Sent data:", dataToSend);
    client.write("\n"); // Add newline as a delimiter
});

client.on("data", function(data) {
    console.log("Received data from server:");
    // If the server is sending back data, decode it here
    // const decodedData = msgpack.decode(data);
    // console.log(decodedData);
});

client.on("close", function() {
    console.log("Connection closed");
});

client.on("error", function(err) {
    console.error("Error:", err);
});
