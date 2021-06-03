import fetch from 'node-fetch';
const https = require('https');

export class DataStore {
  private localDS: any;
  private clusterSecret?: string;
  private isLocal: boolean;
  private headers?: any;
  private ctrlPortUrl: string;
  private ctrlPortAgent: any;

  constructor() {
    this.localDS = {};
    this.clusterSecret = process.env.CLUSTER_SECRET;
    this.isLocal = !this.clusterSecret;
    this.headers = this.clusterSecret ? { [this.clusterSecret]: 'true' } : null;
    this.ctrlPortUrl = 'https://localhost:4142/app/';
    // To avoid failure due to self signed certs
    this.ctrlPortAgent = new https.Agent({
      rejectUnauthorized: false,
    });
  }

  private async request(method: string, url: string, body?: any): Promise<string> {
    const response = await fetch(url, {
      agent: this.ctrlPortAgent,
      method: method,
      headers: this.headers,
      body,
    });
    return await response.text();
  }
  async get(dsKey: string): Promise<string> {
    if (!dsKey) return undefined;
    if (this.isLocal) {
      return dsKey in this.localDS ?
        JSON.stringify(this.localDS[dsKey]) || this.localDS[dsKey].toString() :
        undefined;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}get/${dsKey.toString()}`);
    } catch (e) {
      return e;
    }
  }

  async set(dsKey: string, dsValue: any): Promise<string> {
    if (!dsKey) return 'fail';
    if (this.isLocal) {
      this.localDS[dsKey] = dsValue;
      return 'ok';
    }
    try {
      return await this.request('POST', `${this.ctrlPortUrl}set/${dsKey.toString()}`, dsValue);
    } catch (_) {
      return 'fail';
    }
  }

  async del(dsKey: string): Promise<boolean> {
    if (!dsKey) return false;
    if (this.isLocal) {
      return dsKey in this.localDS ? delete this.localDS[dsKey] : false;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}del/${dsKey.toString()}`) === 'true';
    } catch (_) {
      return false;
    }
  }

  async has(dsKey: string): Promise<boolean> {
    if (!dsKey) return false;
    if (this.isLocal) {
      return dsKey in this.localDS;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}has/${dsKey.toString()}`) === 'true';
    } catch (_) {
      return false;
    }
  }
}
