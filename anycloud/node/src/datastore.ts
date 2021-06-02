require('cross-fetch/polyfill')

class DataStore {
  private localDS: any;
  private ns: string;
  private clusterSecret?: string;
  private isLocal: boolean;
  private headers?: any;
  private ctrlPortUrl: string;

  constructor() {
    this.localDS = {};
    this.ns = 'kv';
    this.clusterSecret = process.env.CLUSTER_SECRET;
    this.isLocal = this.clusterSecret ? false : true;
    this.headers = this.clusterSecret ? {[this.clusterSecret]: 'true'} : null;
    this.ctrlPortUrl = 'https://localhost:4142';
  }

  private async request(method: string, url: string, body?: any): Promise<Response> {
    console.log('localpath', this.isLocal);
    return fetch(url, {
      method: method,
      headers: this.headers,
      body,
    });
  }

  get(dsKey: string): any | Promise<Response> {
    if (this.isLocal) return this.localDS[dsKey];
    return this.request('GET', `${this.ctrlPortUrl}/app/get/${dsKey.toString()}`);
  }

  set(dsKey: string, dsValue: any): any | Promise<Response> {
    if (this.isLocal) {
      return this.localDS[dsKey] = dsValue;
    }
    return this.request('POST', `${this.ctrlPortUrl}/app/set/${dsKey.toString()}`, dsValue);
  }

  del(dsKey: string): boolean | Promise<Response> {
    if (this.isLocal) {
      if (!(dsKey in this.localDS)) return false;
      return delete this.localDS[dsKey];
    }
    return this.request('GET', `${this.ctrlPortUrl}/app/del/${dsKey.toString()}`);
  }

  has(dsKey: string): boolean | Promise<Response> {
    if (this.isLocal) {
      return dsKey in this.localDS;
    }
    return this.request('GET', `${this.ctrlPortUrl}/app/has/${dsKey.toString()}`);
  }
}

export const DS = new DataStore();

// TODO: remove before merge
console.log(DS.set('foo', 'bar'));
console.log(DS.get('foo'));
console.log(DS.has('foo'));
console.log(DS.del('foo'));
console.log(DS.has('foo'));
console.log(DS.set('foo', {foo1: "bar1"}));
console.log(DS.get('foo'));
console.log(DS.del('foo'));
