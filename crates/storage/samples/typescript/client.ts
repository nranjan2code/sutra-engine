import * as net from 'net';
import { encode, decode } from '@msgpack/msgpack';

interface LearnOptions {
    generate_embedding: boolean;
    embedding_model: string | null;
    extract_associations: boolean;
    min_association_confidence: number;
    max_associations_per_concept: number;
    strength: number;
    confidence: number;
}

const defaultOptions: LearnOptions = {
    generate_embedding: true,
    embedding_model: null,
    extract_associations: true,
    min_association_confidence: 0.5,
    max_associations_per_concept: 10,
    strength: 1.0,
    confidence: 1.0
};

class SutraClient {
    private client: net.Socket;
    private host: string;
    private port: number;
    private responseBuf: Buffer = Buffer.alloc(0);
    private pendingResolve: ((value: any) => void) | null = null;
    private expectedLen: number = -1;

    constructor(host: string = 'localhost', port: number = 50051) {
        this.host = host;
        this.port = port;
        this.client = new net.Socket();
    }

    async connect(): Promise<void> {
        return new Promise((resolve, reject) => {
            this.client.connect(this.port, this.host, () => {
                console.log(`Connected to Sutra Storage at ${this.host}:${this.port}`);
                resolve();
            });
            this.client.on('error', reject);
            this.client.on('data', (data: Buffer) => {
                this.responseBuf = Buffer.concat([this.responseBuf, data]);
                this.processIncoming();
            });
        });
    }

    private processIncoming() {
        while (this.responseBuf.length >= 4) {
            if (this.expectedLen === -1) {
                this.expectedLen = this.responseBuf.readUInt32BE(0);
            }

            if (this.expectedLen !== -1 && this.responseBuf.length >= 4 + this.expectedLen) {
                const responsePayload = this.responseBuf.slice(4, 4 + this.expectedLen);
                const response = decode(responsePayload);

                // Remove processed data
                this.responseBuf = this.responseBuf.slice(4 + this.expectedLen);
                this.expectedLen = -1;

                if (this.pendingResolve) {
                    const resolve = this.pendingResolve;
                    this.pendingResolve = null;
                    resolve(response);
                }
            } else {
                break; // Need more data
            }
        }
    }

    private async sendRequest(request: any): Promise<any> {
        const payload = encode(request);
        const lengthBuf = Buffer.alloc(4);
        lengthBuf.writeUInt32BE(payload.length, 0);

        return new Promise((resolve, reject) => {
            if (this.pendingResolve) {
                return reject(new Error('Concurrent request not supported in this simple client'));
            }
            this.pendingResolve = resolve;
            this.client.write(lengthBuf);
            this.client.write(payload);
        });
    }

    async healthCheck(): Promise<any> {
        return this.sendRequest("HealthCheck");
    }

    async learnConcept(namespace: string | null, content: string, options: LearnOptions = defaultOptions): Promise<string> {
        const response = await this.sendRequest({
            LearnConceptV2: {
                namespace,
                content,
                options
            }
        });
        if (response.LearnConceptV2Ok) return response.LearnConceptV2Ok.concept_id;
        throw new Error(response.Error?.message || 'Unknown error');
    }

    async learnWithEmbedding(namespace: string, content: string, embedding: number[], metadata: Record<string, string>): Promise<string> {
        const response = await this.sendRequest({
            LearnWithEmbedding: {
                id: null,
                namespace,
                content,
                embedding,
                metadata,
                timestamp: Math.floor(Date.now() / 1000)
            }
        });
        if (response.LearnConceptV2Ok) return response.LearnConceptV2Ok.concept_id;
        throw new Error(response.Error?.message || 'Unknown error');
    }

    async queryConcept(namespace: string | null, id: string): Promise<any> {
        const response = await this.sendRequest({
            QueryConcept: { namespace, concept_id: id }
        });
        if (response.QueryConceptOk) return response.QueryConceptOk;
        throw new Error(response.Error?.message || 'Not found');
    }

    async deleteConcept(namespace: string, id: string): Promise<string> {
        const response = await this.sendRequest({
            DeleteConcept: { namespace, id }
        });
        if (response.DeleteConceptOk) return response.DeleteConceptOk.id;
        throw new Error(response.Error?.message || 'Delete failed');
    }

    async listRecent(namespace: string, limit: number = 10): Promise<any[]> {
        const response = await this.sendRequest({
            ListRecent: { namespace, limit }
        });
        if (response.ListRecentOk) return response.ListRecentOk.items;
        throw new Error(response.Error?.message || 'List failed');
    }

    async clearCollection(namespace: string): Promise<void> {
        const response = await this.sendRequest({
            ClearCollection: { namespace }
        });
        if (!response.ClearCollectionOk) throw new Error(response.Error?.message || 'Clear failed');
    }

    async getStats(namespace: string | null): Promise<any> {
        const response = await this.sendRequest({
            GetStats: { namespace }
        });
        if (response.StatsOk) return response.StatsOk;
        throw new Error(response.Error?.message || 'Stats failed');
    }

    async flush(): Promise<void> {
        const response = await this.sendRequest("Flush");
        if (response === "FlushOk" || response.FlushOk) return;
        throw new Error(response.Error?.message || 'Flush failed');
    }

    close() {
        this.client.destroy();
    }
}

async function main() {
    const client = new SutraClient();
    try {
        await client.connect();

        console.log('--- Health Check ---');
        console.log(await client.healthCheck());

        const testNSE = 'engine_test_v2';

        console.log(`\n--- Learning in namespace [${testNSE}] ---`);
        const id1 = await client.learnConcept(testNSE, "Sutra Engine is a high-performance knowledge graph.");
        console.log(`Learned concept ID: ${id1}`);

        console.log('\n--- Learning with precomputed embedding ---');
        // Dummy 768-dim vector
        const dummyEmbedding = Array(768).fill(0).map(() => Math.random());
        const id2 = await client.learnWithEmbedding(testNSE, "Context-aware memory enables long-term reasoning.", dummyEmbedding, { source: "manual_upload", priority: "high" });
        console.log(`Learned with embedding ID: ${id2}`);

        console.log('\n--- Flushing storage ---');
        await client.flush();

        console.log('\n--- Querying concept ---');
        const concept = await client.queryConcept(testNSE, id2);
        console.log('Concept Data:', concept);

        console.log('\n--- Listing recent items ---');
        const recent = await client.listRecent(testNSE, 5);
        console.log(`Found ${recent.length} recent items`);
        recent.forEach((item, i) => {
            console.log(`${i + 1}. [${item.id}] ${item.content_preview.substring(0, 50)}...`);
            console.log(`   Attributes:`, item.attributes);
        });

        console.log('\n--- Deleting concept ---');
        await client.deleteConcept(testNSE, id1);
        await client.flush();
        console.log(`Deleted ${id1}`);

        console.log('\n--- Final Stats ---');
        console.log(await client.getStats(testNSE));

    } catch (err) {
        console.error('Error:', err);
    } finally {
        client.close();
    }
}

main();
