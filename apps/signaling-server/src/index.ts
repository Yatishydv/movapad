import express from 'express';
import http from 'http';
import { WebSocketServer, WebSocket } from 'ws';
import cors from 'cors';
import dotenv from 'dotenv';
import { v4 as uuidv4 } from 'uuid';

dotenv.config();

const app = express();
app.use(cors());
app.use(express.json());

const server = http.createServer(app);
const wss = new WebSocketServer({ server });

const PORT = process.env.SIGNALING_PORT || 8080;
const HOST = process.env.SIGNALING_HOST || '0.0.0.0';

app.get('/health', (req, res) => {
    res.json({ status: 'ok', time: new Date().toISOString() });
});

interface Client {
    id: string;
    ws: WebSocket;
    role?: 'desktop' | 'mobile';
    roomId?: string;
}

interface Room {
    id: string;
    pin: string;
    desktop?: Client;
    mobile?: Client;
}

const clients = new Map<string, Client>();
const rooms = new Map<string, Room>();
const roomsByPin = new Map<string, Room>();

// Generate a random 6-digit PIN
function generatePin(): string {
    return Math.floor(100000 + Math.random() * 900000).toString();
}

wss.on('connection', (ws) => {
    const clientId = uuidv4();
    const client: Client = { id: clientId, ws };
    clients.set(clientId, client);

    console.log(`Client connected: ${clientId}. Total clients: ${clients.size}`);

    ws.on('message', (message) => {
        try {
            const data = JSON.parse(message.toString());
            const type = data.type;

            switch (type) {
                case 'CREATE_ROOM': {
                    // Desktop creates a room
                    const roomId = uuidv4();
                    const pin = generatePin();
                    
                    client.role = 'desktop';
                    client.roomId = roomId;

                    const room: Room = { id: roomId, pin, desktop: client };
                    rooms.set(roomId, room);
                    roomsByPin.set(pin, room);

                    ws.send(JSON.stringify({ type: 'ROOM_CREATED', roomId, pin }));
                    console.log(`Desktop created room ${roomId} with PIN ${pin}`);
                    break;
                }
                case 'JOIN_ROOM': {
                    // Mobile joins a room via PIN
                    const pin = data.pin;
                    const room = roomsByPin.get(pin);

                    if (!room || !room.desktop) {
                        ws.send(JSON.stringify({ type: 'ERROR', message: 'Invalid PIN or room expired' }));
                        return;
                    }
                    if (room.mobile) {
                        ws.send(JSON.stringify({ type: 'ERROR', message: 'Room is full' }));
                        return;
                    }

                    client.role = 'mobile';
                    client.roomId = room.id;
                    room.mobile = client;

                    ws.send(JSON.stringify({ type: 'ROOM_JOINED', roomId: room.id }));
                    room.desktop.ws.send(JSON.stringify({ type: 'PEER_CONNECTED' }));
                    console.log(`Mobile joined room ${room.id} with PIN ${pin}`);
                    break;
                }
                case 'offer':
                case 'answer':
                case 'candidate': {
                    // Relay WebRTC signaling messages to the other peer in the room
                    if (!client.roomId) return;
                    const room = rooms.get(client.roomId);
                    if (!room) return;

                    const target = client.role === 'desktop' ? room.mobile : room.desktop;
                    if (target && target.ws.readyState === WebSocket.OPEN) {
                        target.ws.send(JSON.stringify(data));
                    }
                    break;
                }
                default:
                    console.log(`Unknown message type: ${type}`);
            }
        } catch (e) {
            console.error('Invalid message received', e);
        }
    });

    ws.on('close', () => {
        console.log(`Client disconnected: ${clientId}`);
        clients.delete(clientId);

        if (client.roomId) {
            const room = rooms.get(client.roomId);
            if (room) {
                // Notify the other peer
                const target = client.role === 'desktop' ? room.mobile : room.desktop;
                if (target && target.ws.readyState === WebSocket.OPEN) {
                    target.ws.send(JSON.stringify({ type: 'PEER_DISCONNECTED' }));
                }

                // Clean up room
                rooms.delete(room.id);
                roomsByPin.delete(room.pin);
                console.log(`Room ${room.id} destroyed`);
            }
        }
    });
});

server.listen(PORT as number, HOST, () => {
    console.log(`Signaling server listening on http://${HOST}:${PORT}`);
});
