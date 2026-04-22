import { createInitialState } from './state.ts';
import { createErrorPanel } from '../features/errorPanel.ts';
import { createFunctionInput } from '../features/functionInput.ts';
import { createRevolutionPanel } from '../features/revolutionPanel.ts';
import { createRiemannPanel } from '../features/riemannPanel.ts';
import { createTaylorPanel } from '../features/taylorPanel.ts';
import { EngineClient } from '../wasm/engine.ts';

export function createApp(): HTMLElement {
  const state = createInitialState();
  const engine = new EngineClient();
  const app = document.createElement('main');
  app.className = 'app-shell';

  const header = document.createElement('header');
  header.className = 'hero';
  const heroText = document.createElement('div');
  heroText.innerHTML = `
    <p class="eyebrow">Rust + WASM + TypeScript</p>
    <h1>Calculus Engine</h1>
    <p class="hero-copy">The math core, Taylor engine, Riemann geometry, and disk mesh pipeline now exist in Rust. These panels now target the WASM engine wrapper directly.</p>
  `;
  const statusCard = document.createElement('div');
  statusCard.className = 'status-card';
  const statusLabel = document.createElement('span');
  statusLabel.textContent = 'Status';
  const statusValue = document.createElement('strong');
  statusValue.textContent = state.status;
  const statusMessage = document.createElement('p');
  statusMessage.textContent = state.message;
  statusCard.append(statusLabel, statusValue, statusMessage);
  header.append(heroText, statusCard);

  const updateStatus = (status: typeof state.status, message: string) => {
    state.status = status;
    state.message = message;
    statusValue.textContent = status;
    statusMessage.textContent = message;
  };

  const applyExpression = async (expression: string) => {
    updateStatus('loading', 'Applying expression in the WASM engine');
    await engine.setPanelGraphExpression(expression);
    state.expression = expression;
    updateStatus('ready', `Expression loaded: ${expression}`);
    return `Expression parsed and cached in Rust: ${expression}`;
  };

  const clearAppliedExpression = async () => {
    updateStatus('loading', 'Clearing the active calculus source');
    await engine.clearPanelGraphExpression();
    state.expression = '';
    updateStatus('ready', 'No explicit calculus source is selected');
  };

  const analyzeGraphExpression = async (expression: string) => engine.analyzeGraphExpression(expression);
  const sampleGraphCurve = async (
    expression: string,
    request: Parameters<EngineClient['sampleGraphCurve']>[1],
  ) => engine.sampleGraphCurve(expression, request);
  const buildGraphRelationGrid = async (
    expression: string,
    request: Parameters<EngineClient['buildGraphRelationGrid']>[1],
  ) => engine.buildGraphRelationGrid(expression, request);

  const grid = document.createElement('section');
  grid.className = 'panel-grid';
  grid.append(
    createFunctionInput(
      state.expression,
      applyExpression,
      clearAppliedExpression,
      analyzeGraphExpression,
      sampleGraphCurve,
      buildGraphRelationGrid,
    ),
    createRiemannPanel(async (request) => {
      updateStatus('loading', 'Building Riemann geometry');
      const snapshot = await engine.buildRiemannGeometry(request);
      updateStatus('ready', 'Riemann geometry updated');
      return snapshot;
    }),
    createTaylorPanel(async (request) => {
      updateStatus('loading', 'Building Taylor series');
      const snapshot = await engine.buildTaylorSeries(request);
      updateStatus('ready', 'Taylor series updated');
      return snapshot;
    }),
    createRevolutionPanel(async (request) => {
      updateStatus('loading', 'Building disk mesh');
      const snapshot = await engine.buildDiskMesh(request);
      updateStatus('ready', 'Disk mesh updated');
      return snapshot;
    }),
    createErrorPanel(async (request) => {
      updateStatus('loading', 'Building numerical error series');
      const snapshot = await engine.buildRiemannErrorSeries(request);
      updateStatus('ready', 'Error series updated');
      return snapshot;
    }),
  );

  app.append(header, grid);
  return app;
}
