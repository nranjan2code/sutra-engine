import * as net from 'net';
import msgpack from 'msgpack5';

const { encode, decode } = msgpack();

/**
 * Basic TypeScript Client for Sutra Engine (TCP)
 */
class SutraEngineClient {
    private client: net.Socket;
    private host: string;
    private port: number;

    constructor(host: string = 'localhost', port: number = 50051) {
        this.host = host;
        this.port = port;
        this.client = new net.Socket();
    }

    async connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            this.client.connect(this.port, this.host, () => {
                console.log(`Connected to Sutra Engine at ${this.host}:${this.port}`);
                resolve();
            });
            this.client.on('error', (err) => reject(err));
        });
    }

    async sendRequest(request: any): Promise<any> {
        return new Promise((resolve, reject) => {
            const payload = encode(request);
            const header = Buffer.allocUnsafe(4);
            header.writeUInt32BE(payload.length, 0);

            this.client.write(Buffer.concat([header, payload]));

            this.client.once('data', (data) => {
                const responseLength = data.readUInt32BE(0);
                const responsePayload = data.slice(4, 4 + responseLength);
                resolve(decode(responsePayload));
            });
        });
    }

    async learn(content: string) {
        return this.sendRequest({
            LearnConceptV2: {
                content,
                options: {
                    generate_embedding: false,
                    embedding_model: null,
                    extract_associations: true,
                    min_association_confidence: 0.5,
                    max_associations_per_concept: 10,
                    strength: 1.0,
                    confidence: 1.0
                }
            }
        });
    }

    async search(query: string, limit: number = 5) {
        return this.sendRequest({
            TextSearch: { query, limit }
        });
    }

    close() {
        this.client.destroy();
    }
}

// Demo Usage
async function run() {
    const sutra = new SutraEngineClient();
    try {
        await sutra.connect();
        console.log("Ingesting knowledge...");
        const result = await sutra.learn("TypeScript is a strongly typed programming language.");
        console.log("Ingested:", result);

        console.log("Searching...");
        const searchResult = await sutra.search("programming");
        console.log("Results:", JSON.stringify(searchResult, null, 2));
    } catch (e) {
        console.error("Demo failed:", e);
    } finally {
        sutra.close();
    }
}

run().catch(console.error);
