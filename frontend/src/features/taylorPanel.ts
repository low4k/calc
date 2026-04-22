import type { TaylorBuildRequest, TaylorSeriesSnapshot } from '../wasm/engine.ts';
import { createCanvasPlot, renderCartesianPlot } from '../render2d/canvasPlot.ts';

export function createTaylorPanel(
  onRun: (request: TaylorBuildRequest) => Promise<TaylorSeriesSnapshot>,
): HTMLElement {
  const section = document.createElement('section');
  section.className = 'panel';
  section.innerHTML = '<h2>Taylor Series</h2>';

  const form = document.createElement('div');
  form.className = 'control-grid';
  const center = createNumberControl('Center', 0);
  const degree = createNumberControl('Degree', 5, 0);
  const start = createNumberControl('Start', -1);
  const end = createNumberControl('End', 1);
  const sampleCount = createNumberControl('Samples', 25, 2);
  form.append(center.label, degree.label, start.label, end.label, sampleCount.label);

  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = 'Build Taylor Series';
  button.className = 'panel-button';

  const showFunction = document.createElement('input');
  showFunction.type = 'checkbox';
  showFunction.checked = true;
  const showPolynomial = document.createElement('input');
  showPolynomial.type = 'checkbox';
  showPolynomial.checked = true;
  const showError = document.createElement('input');
  showError.type = 'checkbox';
  showError.checked = true;
  const toggles = document.createElement('div');
  toggles.className = 'toggle-row';
  toggles.append(
    wrapCheckbox('Original function', showFunction),
    wrapCheckbox('Taylor polynomial', showPolynomial),
    wrapCheckbox('Error plot', showError),
  );

  const mainCanvas = createCanvasPlot();
  const errorCanvas = createCanvasPlot();
  errorCanvas.classList.add('plot-canvas-compact');
  const mainFrame = document.createElement('div');
  mainFrame.className = 'plot-frame';
  mainFrame.appendChild(mainCanvas);
  const errorFrame = document.createElement('div');
  errorFrame.className = 'plot-frame';
  errorFrame.appendChild(errorCanvas);

  const result = document.createElement('p');
  result.className = 'panel-copy panel-result';
  result.textContent = 'Generate Taylor coefficients and sampled comparison data.';

  button.addEventListener('click', () => {
    void (async () => {
      button.disabled = true;
      result.textContent = 'Building Taylor series...';
      try {
        const snapshot = await onRun({
          center: Number(center.input.value),
          degree: Number(degree.input.value),
          start: Number(start.input.value),
          end: Number(end.input.value),
          sampleCount: Number(sampleCount.input.value),
        });
        renderTaylorSnapshot(mainCanvas, errorCanvas, snapshot, {
          showFunction: showFunction.checked,
          showPolynomial: showPolynomial.checked,
          showError: showError.checked,
        });
        const leading = Array.from(snapshot.coefficients.slice(0, 4)).map((value) => value.toFixed(4)).join(', ');
        const maxError = Math.max(...snapshot.absoluteError);
        result.textContent = `coeffs=[${leading}${snapshot.coefficients.length > 4 ? ', ...' : ''}] samples=${snapshot.sampleX.length} max_error=${maxError.toExponential(3)}`;
      } catch (error) {
        result.textContent = error instanceof Error ? error.message : String(error);
      } finally {
        button.disabled = false;
      }
    })();
  });

  section.append(form, toggles, button, mainFrame, errorFrame, result);
  return section;
}

function renderTaylorSnapshot(
  mainCanvas: HTMLCanvasElement,
  errorCanvas: HTMLCanvasElement,
  snapshot: TaylorSeriesSnapshot,
  toggles: { showFunction: boolean; showPolynomial: boolean; showError: boolean },
): void {
  const xValues = Array.from(snapshot.sampleX);
  renderCartesianPlot(mainCanvas, {
    title: 'Taylor Approximation',
    xLabel: 'x',
    yLabel: 'value',
    lineSeries: [
      toggles.showFunction
        ? {
            color: '#a33f27',
            width: 2.6,
            label: 'f(x)',
            points: xValues.map((x, index) => ({ x, y: snapshot.functionValues[index] })),
          }
        : null,
      toggles.showPolynomial
        ? {
            color: '#2a6f83',
            width: 2.1,
            label: 'P_n(x)',
            points: xValues.map((x, index) => ({ x, y: snapshot.polynomialValues[index] })),
          }
        : null,
    ].filter(Boolean) as Array<{
      color: string;
      width: number;
      label: string;
      points: { x: number; y: number }[];
    }>,
  });

  renderCartesianPlot(errorCanvas, {
    title: 'Taylor Absolute Error',
    xLabel: 'x',
    yLabel: '|f(x) - P_n(x)|',
    lineSeries: toggles.showError
      ? [
          {
            color: '#5a2d17',
            width: 2.1,
            label: 'absolute error',
            points: xValues.map((x, index) => ({ x, y: snapshot.absoluteError[index] })),
          },
        ]
      : [],
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
