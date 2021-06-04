import fetch from 'node-fetch';
const https = require('https');

class DataStore {
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

  static isInvalidKey(key: string): boolean {
    return !key || typeof(key) !== 'string';
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
    if (DataStore.isInvalidKey(dsKey)) return undefined;
    if (this.isLocal) {
      if (dsKey in this.localDS) {
        const val = this.localDS[dsKey];
        return JSON.parse(val) || val;
      } else {
        return undefined;
      }
    }
    try {
      const dsValue = await this.request('GET', `${this.ctrlPortUrl}get/${dsKey}`);
      return JSON.parse(dsValue) || dsValue;
    } catch (e) {
      return e;
    }
  }

  async set(dsKey: string, dsValue: any): Promise<string> {
    if (DataStore.isInvalidKey(dsKey)) return 'fail';
    const maybeStringifiedValue = (function () {
      try {
        return JSON.stringify(dsValue);
      } catch (_) {
        return dsValue;
      }
    })();
    if (this.isLocal) {
      this.localDS[dsKey] = maybeStringifiedValue;
      return 'ok';
    }
    try {
      return await this.request('POST', `${this.ctrlPortUrl}set/${dsKey}`, maybeStringifiedValue);
    } catch (_) {
      return 'fail';
    }
  }

  async del(dsKey: string): Promise<boolean> {
    if (DataStore.isInvalidKey(dsKey)) return false;
    if (this.isLocal) {
      return dsKey in this.localDS ? delete this.localDS[dsKey] : false;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}del/${dsKey}`) === 'true';
    } catch (_) {
      return false;
    }
  }

  async has(dsKey: string): Promise<boolean> {
    if (DataStore.isInvalidKey(dsKey)) return false;
    if (this.isLocal) {
      return dsKey in this.localDS;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}has/${dsKey}`) === 'true';
    } catch (_) {
      return false;
    }
  }
}

const ds = new DataStore();

export const DS = new Proxy({}, {
  get: async (_, dsKey: string): Promise<string> => {
    return await ds.get(dsKey);
  },
  set: (_, dsKey: string, dsValue: any): boolean => {
    if (DataStore.isInvalidKey(dsKey)) {
      return false;
    }
    ds.set(dsKey, dsValue);
    return true;
  },
  deleteProperty: (_, dsKey: string): boolean => {
    if (DataStore.isInvalidKey(dsKey)) {
      return false;
    }
    ds.del(dsKey);
    return true;
  }
});
