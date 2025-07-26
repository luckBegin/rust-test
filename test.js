const net = require('net');
const readline = require('readline');

const HOST = '127.0.0.1';
const PORT = 12345; // 改成你的服务端端口

const client = new net.Socket();

// 连接服务器
client.connect(PORT, HOST, () => {
    console.log(`✅ Connected to: ${HOST}:${PORT}`);
    promptInput();
});

// 监听服务端消息
client.on('data', (data) => {
    console.log(`📨 Received: ${data.toString().trim()}`);
});

// 断开连接
client.on('close', () => {
    console.log('❌ Connection closed');
    process.exit(0);
});

// 连接出错
client.on('error', (err) => {
    console.error(`🚫 Error: ${err.message}`);
    process.exit(1);
});

// 读取用户输入
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

function promptInput() {
    rl.question('💬 Send: ', (input) => {
        if (input.toLowerCase() === 'exit') {
            client.end();
            rl.close();
            return;
        }

        client.write(input + '\n');
        promptInput();
    });
}