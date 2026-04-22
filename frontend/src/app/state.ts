import type { AppState } from './types.ts';

export function createInitialState(): AppState {
  return {
    expression: 'sin(x)',
    status: 'idle',
    message: 'Bootstrap shell ready',
  };
}
