import { v4 as uuid } from 'uuid';

export const genName = () => '_' + uuid().replace(/-/g, '_');

export const TODO = (task?: string) => { throw new Error(`TODO${task !== undefined ? ': ' + task : ''}`) };
