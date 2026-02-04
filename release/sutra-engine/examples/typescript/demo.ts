import * as net from 'net';
import { encode, decode } from '@msgpack/msgpack';

/**
 * Professional Sutra Engine Client (TypeScript)
 * This client demonstrates the high-performance binary protocol
 * supporting multi-tenant namespaces and CRUD operations.
 */

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
                console.log(`🚀 Connected to Sutra Engine at ${this.host}:${this.port}`);
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

    /**
     * Ingest a new concept into the reasoning core.
     */
    async learn(content: string, namespace: string = 'default'): Promise<string> {
        const response = await this.sendRequest({
            LearnConceptV2: {
                namespace,
                content,
                options: defaultOptions
            }
        });
        if (response.LearnConceptV2Ok) return response.LearnConceptV2Ok.concept_id;
        throw new Error(response.Error?.message || 'Learning failed');
    }

    /**
     * Search for a specific concept and its relationships.
     */
    async query(id: string, namespace: string = 'default'): Promise<any> {
        const response = await this.sendRequest({
            QueryConcept: { namespace, concept_id: id }
        });
        if (response.QueryConceptOk) return response.QueryConceptOk;
        throw new Error(response.Error?.message || 'Query failed');
    }

    /**
     * Delete a concept and its edges.
     */
    async delete(id: string, namespace: string = 'default'): Promise<void> {
        const response = await this.sendRequest({
            DeleteConcept: { namespace, id }
        });
        if (!response.DeleteConceptOk) throw new Error(response.Error?.message || 'Delete failed');
    }

    /**
     * List recently ingested items in a namespace.
     */
    async listRecent(namespace: string = 'default', limit: number = 5): Promise<any[]> {
        const response = await this.sendRequest({
            ListRecent: { namespace, limit }
        });
        if (response.ListRecentOk) return response.ListRecentOk.items;
        throw new Error(response.Error?.message || 'List failed');
    }

    /**
     * Force a disk flush to ensure durability.
     */
    async flush(): Promise<void> {
        const response = await this.sendRequest("Flush");
        if (response === "FlushOk" || response.FlushOk) return;
        throw new Error('Flush failed');
    }

    /**
     * Get system-wide or namespace-specific statistics.
     */
    async getStats(namespace: string | null = null): Promise<any> {
        const response = await this.sendRequest({
            GetStats: { namespace }
        });
        if (response.StatsOk) return response.StatsOk;
        throw new Error('Stats failed');
    }

    close() {
        this.client.destroy();
    }
}

async function main() {
    const client = new SutraClient();
    try {
        await client.connect();

        const NS = 'production_core';

        console.log('\n--- 1. Multi-Tenant Ingestion ---');
        const cid = await client.learn("Artificial Intelligence is transforming reasoning engines.", NS);
        console.log(`✅ Learned concept: ${cid} in namespace: ${NS}`);

        console.log('\n--- 2. Durability Check (Flush) ---');
        await client.flush();
        console.log('✅ Changes persisted to disk.');

        console.log('\n--- 3. Explanatory Query ---');
        const data = await client.query(cid, NS);
        console.log('Concept Data:', data);

        console.log('\n--- 4. List Recent (Visibility) ---');
        const recent = await client.listRecent(NS);
        recent.forEach(item => console.log(` - [${item.id}] ${item.content_preview}`));

        console.log('\n--- 5. Cleanup (Delete) ---');
        await client.delete(cid, NS);
        console.log(`✅ Deleted concept: ${cid}`);

        console.log('\n--- 6. Statistics ---');
        const stats = await client.getStats(NS);
        console.log('Engine Stats:', stats);

    } catch (err) {
        console.error('❌ Error:', err);
    } finally {
        client.close();
    }
}

main();
