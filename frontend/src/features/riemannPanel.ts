import type { RiemannBuildRequest, RiemannGeometrySnapshot, RiemannMethod } from '../wasm/engine.ts';
import { createCanvasPlot, renderCartesianPlot } from '../render2d/canvasPlot.ts';

export function createRiemannPanel(
  onRun: (request: RiemannBuildRequest) => Promise<RiemannGeometrySnapshot>,
): HTMLElement {
  const section = document.createElement('section');
  section.className = 'panel';
  section.innerHTML = '<h2>Riemann Sums</h2>';

  const form = document.createElement('div');
  form.className = 'control-grid';

  const start = createNumberControl('Start', 0);
  const end = createNumberControl('End', Math.PI);
  const subdivisions = createNumberControl('Subdivisions', 16, 1);
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
  form.append(start.label, end.label, subdivisions.label, wrapControl('Method', method));

  const button = document.createElement('button');
  button.type = 'button';
  button.textContent = 'Build Riemann Geometry';
  button.className = 'panel-button';

  const showShapes = document.createElement('input');
  showShapes.type = 'checkbox';
  showShapes.checked = true;
  const toggles = document.createElement('div');
  toggles.className = 'toggle-row';
  toggles.append(wrapCheckbox('Show primitives', showShapes));

  const canvas = createCanvasPlot();
  const frame = document.createElement('div');
  frame.className = 'plot-frame';
  frame.appendChild(canvas);

  const result = document.createElement('p');
  result.className = 'panel-copy panel-result';
  result.textContent = 'Run the current expression through the Riemann geometry pipeline.';

  button.addEventListener('click', () => {
    void (async () => {
      button.disabled = true;
      result.textContent = 'Building Riemann geometry...';
      try {
        const snapshot = await onRun({
          start: Number(start.input.value),
          end: Number(end.input.value),
          subdivisions: Number(subdivisions.input.value),
          method: method.value as RiemannMethod,
        });
        renderRiemannSnapshot(canvas, snapshot, showShapes.checked);
        result.textContent = `approx=${snapshot.approximation.toFixed(6)} ref=${snapshot.referenceValue.toFixed(6)} error=${snapshot.absoluteError.toExponential(3)} rectangles=${snapshot.rectangles.length / 5} trapezoids=${snapshot.trapezoids.length / 6}`;
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

function renderRiemannSnapshot(canvas: HTMLCanvasElement, snapshot: RiemannGeometrySnapshot, showShapes: boolean): void {
  const rectangles = [];
  for (let index = 0; index < snapshot.rectangles.length; index += 5) {
    rectangles.push({
      x: snapshot.rectangles[index],
      y: snapshot.rectangles[index + 1],
      width: snapshot.rectangles[index + 2],
      height: snapshot.rectangles[index + 3],
      fill: 'rgba(198, 102, 63, 0.28)',
      stroke: 'rgba(90, 45, 23, 0.55)',
    });
  }

  const polygons = [];
  for (let index = 0; index < snapshot.trapezoids.length; index += 6) {
    const x0 = snapshot.trapezoids[index];
    const y0 = snapshot.trapezoids[index + 1];
    const x1 = snapshot.trapezoids[index + 2];
    const y1 = snapshot.trapezoids[index + 3];
    const baseline = snapshot.trapezoids[index + 4];
    polygons.push({
      points: [
        { x: x0, y: baseline },
        { x: x0, y: y0 },
        { x: x1, y: y1 },
        { x: x1, y: baseline },
      ],
      fill: 'rgba(163, 63, 39, 0.22)',
      stroke: 'rgba(90, 45, 23, 0.6)',
    });
  }

  const lineSeries = polygons.length > 0
    ? [{
        color: '#5a2d17',
        points: polygons.flatMap((polygon) => [polygon.points[1], polygon.points[2]]),
      }]
    : [{
        color: '#5a2d17',
        points: rectangles.flatMap((rectangle) => [
          { x: rectangle.x, y: rectangle.y },
          { x: rectangle.x + rectangle.width, y: rectangle.y },
        ]),
      }];

  renderCartesianPlot(canvas, {
    title: 'Riemann Approximation',
    xLabel: 'x',
    yLabel: 'height',
    rectangles: showShapes ? rectangles : [],
    polygons: showShapes ? polygons : [],
    lineSeries,
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
