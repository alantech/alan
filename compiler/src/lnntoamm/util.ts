import { v4 as uuid } from 'uuid';

export const genName = () => uuid().replace(/-/g, '_');

export const TODO = (task: string) => { throw new Error(`TODO: ${task}`) };
