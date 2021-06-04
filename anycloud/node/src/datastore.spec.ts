import { DataStore } from './datastore';

let ds;

describe('Datastore class local test', () => {

  beforeEach(() => {
    ds = new DataStore();
  });

  describe('set', () => {
    test('should fail if no key passed', async () => {
      expect(await ds.set()).toEqual('fail');
    });

    test('should set undefined value', async () => {
      expect(await ds.set('key')).toEqual('ok');
    });

    test('should set string value', async () => {
      expect(await ds.set('key', 'hello world')).toEqual('ok');
    });

    test('should set number value', async () => {
      expect(await ds.set('key', 123)).toEqual('ok');
    });

    test('should set object value', async () => {
      expect(await ds.set('key', { a: 1 })).toEqual('ok');
    });
  });

  describe('get', () => {
    test('should return undefined if no key passed', async () => {
      expect(await ds.get()).toEqual(undefined);
    });

    test('should return undefined if key not found', async () => {
      expect(await ds.get('key')).toEqual(undefined);
    });

    test('should get string value setted', async () => {
      const key = 'key';
      const val = 'hello world';
      await ds.set(key, val);
      expect(await ds.get(key)).toEqual(val);
    });

    test('should get number value setted', async () => {
      const key = 'key';
      const val = 123;
      await ds.set(key, val);
      expect(await ds.get(key)).toEqual(val);
    });

    test('should get object value setted', async () => {
      const key = 'key';
      const val = { a: 1 };
      await ds.set(key, val);
      expect(await ds.get(key)).toEqual(val);
    });
  });

  describe('has', () => {
    test('should return false if no key passed', async () => {
      expect(await ds.has()).toEqual(false);
    });

    test('should return false if key not found', async () => {
      expect(await ds.has('key')).toEqual(false);
    });

    test('should return true if value setted', async () => {
      const key = 'key';
      const val = 'hello world';
      await ds.set(key, val);
      expect(await ds.has(key)).toEqual(true);
    });
  });

  describe('del', () => {
    test('should return false if no key passed', async () => {
      expect(await ds.del()).toEqual(false);
    });

    test('should return false if key not found', async () => {
      expect(await ds.del('key')).toEqual(false);
    });

    test('should return true if value found and deleted', async () => {
      const key = 'key';
      const val = 'hello world';
      await ds.set(key, val);
      expect(await ds.del(key)).toEqual(true);
    });

    test('should delete value', async () => {
      const key = 'key';
      const val = 'hello world';
      await ds.set(key, val);
      await ds.del(key);
      expect(await ds.get(key)).toEqual(undefined);
      expect(await ds.has(key)).toEqual(false);
    });
  });
});
