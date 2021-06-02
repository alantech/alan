import fetch from 'cross-fetch';
const https = require('https');

export class DataStore {
  private localDS: any;
  private clusterSecret?: string;
  private isLocal: boolean;
  private headers?: any;
  private ctrlPortUrl: string;

  constructor() {
    this.localDS = {};
    this.clusterSecret = process.env.CLUSTER_SECRET;
    this.isLocal = this.clusterSecret ? false : true;
    this.headers = this.clusterSecret ? { [this.clusterSecret]: 'true' } : null;
    this.ctrlPortUrl = 'https://localhost:4142/app/';
    // To avoid failure due to self signed certs
    https.globalAgent.options.rejectUnauthorized = false;
  }

  private async request(method: string, url: string, body?: any): Promise<Response> {
    return fetch(url, {
      method: method,
      headers: this.headers,
      body,
    });
  }

  async get(dsKey: string): Promise<string> {
    if (this.isLocal) return Promise.resolve(this.localDS[dsKey].toString());
    try {
      const res = await this.request('GET', `${this.ctrlPortUrl}get/${dsKey.toString()}`);
      return await res.text();
    } catch (e) {
      return Promise.resolve(e);
    }
  }

  async set(dsKey: string, dsValue: any): Promise<string> {
    if (this.isLocal) {
      this.localDS[dsKey] = dsValue;
      return Promise.resolve('ok');
    }
    try {
      const res = await this.request('POST', `${this.ctrlPortUrl}set/${dsKey.toString()}`, dsValue);
      return await res.text();
    } catch (_) {
      return Promise.resolve('fail');
    }
  }

  async del(dsKey: string): Promise<boolean> {
    if (this.isLocal) {
      if (!(dsKey in this.localDS)) return false;
      return Promise.resolve(delete this.localDS[dsKey]);
    }
    try {
      const res = await this.request('GET', `${this.ctrlPortUrl}del/${dsKey.toString()}`);
      return !!(await res.text());
    } catch (_) {
      return Promise.resolve(false);
    }
  }

  async has(dsKey: string): Promise<boolean> {
    if (this.isLocal) {
      return Promise.resolve(dsKey in this.localDS);
    }
    try {
      const res = await this.request('GET', `${this.ctrlPortUrl}has/${dsKey.toString()}`);
      return !!(await res.text());
    } catch (_) {
      return Promise.resolve(false);
    }
  }
}

