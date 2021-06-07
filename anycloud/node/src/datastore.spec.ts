import { datastore } from './datastore';

describe('Datastore class local test', () => {
  const key = 'foo';
  const strVal = 'hello world';
  const numberVal = 123;
  const objVal = { bar: 'baz' }
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
