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
    return !key || typeof (key) !== 'string';
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

  async get(dsKey: string): Promise<string | Error> {
    if (DataStore.isInvalidKey(dsKey)) return new Error('Invalid key');
    let dsValue;
    if (this.isLocal) {
      if (dsKey in this.localDS) {
        dsValue = this.localDS[dsKey];
      } else {
        return undefined;
      }
    } else {
      try {
        dsValue = await this.request('GET', `${this.ctrlPortUrl}get/${dsKey}`);
      } catch (e) {
        return new Error(e);
      }
    }
    try {
      return JSON.parse(dsValue);
    } catch (_) {
      return dsValue === '<key not found>' ? undefined : dsValue;
    }
  }

  async set(dsKey: string, dsValue: any): Promise<boolean | Error> {
    if (DataStore.isInvalidKey(dsKey)) return new Error('Invalid key');
    const maybeStringifiedValue = (function () {
      try {
        return JSON.stringify(dsValue);
      } catch (_) {
        return dsValue;
      }
    })();
    if (this.isLocal) {
      this.localDS[dsKey] = maybeStringifiedValue;
      return true;
    }
    try {
      await this.request('POST', `${this.ctrlPortUrl}set/${dsKey}`, maybeStringifiedValue)
      return true;
    } catch (e) {
      return new Error(e);
    }
  }

  async del(dsKey: string): Promise<boolean | Error> {
    if (DataStore.isInvalidKey(dsKey)) return new Error('Invalid key');
    if (this.isLocal) {
      return dsKey in this.localDS ? delete this.localDS[dsKey] : false;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}del/${dsKey}`) === 'true';
    } catch (e) {
      return new Error(e);
    }
  }

  async has(dsKey: string): Promise<boolean | Error> {
    if (DataStore.isInvalidKey(dsKey)) return new Error('Invalid key');
    if (this.isLocal) {
      return dsKey in this.localDS;
    }
    try {
      return await this.request('GET', `${this.ctrlPortUrl}has/${dsKey}`) === 'true';
    } catch (e) {
      return new Error(e);
    }
  }
}

const dsHandler = () => {
  const ds = new DataStore();
  return {
    get: async (_, dsKey: string): Promise<string> => {
      if (DataStore.isInvalidKey(dsKey)) {
        return undefined;
      }
      const response = await ds.get(dsKey);
      return response instanceof Error ? undefined : response;
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
  };
};

export const datastore = new Proxy({}, dsHandler());
