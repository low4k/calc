export type EngineStatus = 'idle' | 'loading' | 'ready' | 'error';

export interface AppState {
  expression: string;
  status: EngineStatus;
  message: string;
}
