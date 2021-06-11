import { datastore, ds } from './datastore';

const key = 'foo';
const strVal = 'hello world';
const numberVal = 123;
const objVal = { bar: 'baz' }

describe('Datastore class local test', () => {
  beforeEach(() => {
    datastore.del(key);
  });

  describe('set', () => {
    test('should set string value', async () => {
      expect(await datastore.set(key, strVal)).toEqual(true);
    });

    test('should set number value', async () => {
      expect(await datastore.set(key, numberVal)).toEqual(true);
    });

    test('should set object value', async () => {
      expect(await datastore.set(key, objVal)).toEqual(true);
    });
  });

  describe('get', () => {
    test('should return undefined if key not found', async () => {
      expect(await datastore.get(key)).toEqual(undefined);
    });

    test('should get string value setted', async () => {
      await datastore.set(key, strVal);
      expect(await datastore.get(key)).toEqual(strVal);
    });

    test('should get number value setted', async () => {
      await datastore.set(key, numberVal);
      expect(await datastore.get(key)).toEqual(numberVal);
    });

    test('should get object value setted', async () => {
      await datastore.set(key, objVal);
      expect(await datastore.get(key)).toEqual(objVal);
    });
  });

  describe('has', () => {
    test('should return false if key not found', async () => {
      expect(await datastore.has(key)).toEqual(false);
    });

    test('should return true if value exist after setted', async () => {
      await datastore.set(key, strVal);
      expect(await datastore.has(key)).toEqual(true);
    });
  });

  describe('del', () => {
    test('should return false if key not found', async () => {
      expect(await datastore.del(key)).toEqual(false);
    });

    test('should return true if value found and deleted', async () => {
      await datastore.set(key, strVal);
      expect(await datastore.del(key)).toEqual(true);
    });

    test('should delete value', async () => {
      await datastore.set(key, strVal);
      await datastore.del(key);
      expect(await datastore.get(key)).toEqual(undefined);
      expect(await datastore.has(key)).toEqual(false);
    });
  });
});

describe('Datastore proxy object local test', () => {
  beforeEach(() => {
    delete ds[key];
  });

  describe('set', () => {
    test('should set string value', () => {
      expect(ds[key] = strVal).toEqual(strVal);
    });

    test('should set number value', () => {
      expect(ds[key] = numberVal).toEqual(numberVal);
    });

    test('should set object value', () => {
      expect(ds[key] = objVal).toEqual(objVal);
    });
  });

  describe('get', () => {
    test('should return undefined if key not found', async () => {
      expect(await ds[key]).toEqual(undefined);
    });

    test('should get string value setted', async () => {
      ds[key] = strVal;
      expect(await ds[key]).toEqual(strVal);
    });

    test('should get number value setted', async () => {
      ds[key] = numberVal;
      expect(await ds[key]).toEqual(numberVal);
    });

    test('should get object value setted', async () => {
      ds[key] = objVal;
      expect(await ds[key]).toEqual(objVal);
    });
  });

  describe('del', () => {
    test('should return true if key not found', () => {
      expect(delete ds[key]).toEqual(true);
    });

    test('should return true if value found and deleted', () => {
      ds[key] = objVal;
      expect(delete ds[key]).toEqual(true);
    });

    test('should delete value', async () => {
      ds[key] = objVal;
      delete ds[key];
      expect(await ds[key]).toEqual(undefined);
    });
  });
});
