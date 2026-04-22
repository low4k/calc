import type { RiemannErrorSeriesRequest, RiemannErrorSeriesSnapshot, RiemannMethod } from '../wasm/engine.ts';
import { createCanvasPlot, renderCartesianPlot } from '../render2d/canvasPlot.ts';

export function createErrorPanel(
  onRun: (request: RiemannErrorSeriesRequest) => Promise<RiemannErrorSeriesSnapshot>,
): HTMLElement {
  const section = document.createElement('section');
  section.className = 'panel';
  section.innerHTML = '<h2>Error Visualization</h2>';

  const form = document.createElement('div');
  form.className = 'control-grid';
  const start = createNumberControl('Start', 0);
  const end = createNumberControl('End', Math.PI);
  const counts = createTextControl('Counts', '4,8,16,32,64');
  const method = document.createElement('select');
  for (const value of ['left', 'right', 'midpoint', 'trapezoid'] satisfies RiemannMethod[]) {
    const option = document.createElement('option');
    option.value = value;
    option.textContent = value;
    if (value === 'midpoint') {
      option.selected = true;
    }
    method.appendChild(option);
  }

  form.append(start.label, end.label, counts.label, wrapControl('Method', method));

  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = 'Build Error Series';
  button.className = 'panel-button';

  const logScale = document.createElement('input');
  logScale.type = 'checkbox';
  const toggles = document.createElement('div');
  toggles.className = 'toggle-row';
  toggles.append(wrapCheckbox('Log10 error scale', logScale));

  const canvas = createCanvasPlot();
  const frame = document.createElement('div');
  frame.className = 'plot-frame';
  frame.appendChild(canvas);

  const result = document.createElement('p');
  result.className = 'panel-copy panel-result';
  result.textContent = 'Compare convergence as subdivision count increases.';

  button.addEventListener('click', () => {
    void (async () => {
      button.disabled = true;
      result.textContent = 'Building error series...';
      try {
        const parsedCounts = counts.input.value
          .split(',')
          .map((value) => Number(value.trim()))
          .filter((value) => Number.isFinite(value) && value > 0)
          .map((value) => Math.floor(value));
        const snapshot = await onRun({
          start: Number(start.input.value),
          end: Number(end.input.value),
          method: method.value as RiemannMethod,
          counts: parsedCounts,
        });
        renderErrorSnapshot(canvas, snapshot, logScale.checked);
        const pairs = Array.from(snapshot.counts, (count, index) => `${count}:${snapshot.absoluteErrors[index].toExponential(2)}`);
        result.textContent = `ref=${snapshot.referenceValue.toFixed(6)} errors=${pairs.join('  ')}`;
      } catch (error) {
        result.textContent = error instanceof Error ? error.message : String(error);
      } finally {
        button.disabled = false;
      }
    })();
  });

  section.append(form, toggles, button, frame, result);
  return section;
}

function renderErrorSnapshot(
  canvas: HTMLCanvasElement,
  snapshot: RiemannErrorSeriesSnapshot,
  useLogScale: boolean,
): void {
  renderCartesianPlot(canvas, {
    title: useLogScale ? 'Riemann Error Convergence (log10)' : 'Riemann Error Convergence',
    xLabel: 'subdivisions',
    yLabel: useLogScale ? 'log10(error)' : 'absolute error',
    lineSeries: [
      {
        color: '#a33f27',
        width: 2.5,
        label: useLogScale ? 'log error' : 'absolute error',
        points: Array.from(snapshot.counts, (count, index) => ({
          x: count,
          y: useLogScale
            ? Math.log10(Math.max(snapshot.absoluteErrors[index], Number.EPSILON))
            : snapshot.absoluteErrors[index],
        })),
      },
    ],
  });
}

function createNumberControl(labelText: string, value: number, min?: number) {
  const input = document.createElement('input');
  input.type = 'number';
  input.value = String(value);
  if (min !== undefined) {
    input.min = String(min);
  }
  return { label: wrapControl(labelText, input), input };
}

function createTextControl(labelText: string, value: string) {
  const input = document.createElement('input');
  input.type = 'text';
  input.value = value;
  return { label: wrapControl(labelText, input), input };
}

function wrapControl(text: string, control: HTMLElement): HTMLLabelElement {
  const label = document.createElement('label');
  const span = document.createElement('span');
  span.textContent = text;
  label.append(span, control);
  return label;
}

function wrapCheckbox(text: string, input: HTMLInputElement): HTMLLabelElement {
  const label = document.createElement('label');
  label.className = 'toggle';
  const span = document.createElement('span');
  span.textContent = text;
  label.append(input, span);
  return label;
}
