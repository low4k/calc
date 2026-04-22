export interface PlotPoint {
  x: number;
  y: number;
}

export interface LineSeries {
  points: PlotPoint[];
  color: string;
  width?: number;
  label?: string;
  dash?: number[];
}

export interface FilledPolygon {
  points: PlotPoint[];
  fill: string;
  stroke?: string;
}

export interface PlotRect {
  x: number;
  y: number;
  width: number;
  height: number;
  fill: string;
  stroke?: string;
  anchor?: 'baseline' | 'corner';
}

export interface PlotOptions {
  lineSeries?: LineSeries[];
  polygons?: FilledPolygon[];
  rectangles?: PlotRect[];
  xDomain?: [number, number];
  yDomain?: [number, number];
  title?: string;
  xLabel?: string;
  yLabel?: string;
}

const PLOT_WIDTH = 960;
const PLOT_HEIGHT = 420;
const PADDING = { top: 18, right: 18, bottom: 34, left: 48 };

export function createCanvasPlot(): HTMLCanvasElement {
  const canvas = document.createElement('canvas');
  canvas.width = PLOT_WIDTH;
  canvas.height = PLOT_HEIGHT;
  canvas.className = 'plot-canvas';
  return canvas;
}

export function renderCartesianPlot(canvas: HTMLCanvasElement, options: PlotOptions): void {
  const context = canvas.getContext('2d');
  if (!context) {
    return;
  }

  const { xDomain, yDomain } = resolveDomains(options);
  const plotWidth = canvas.width - PADDING.left - PADDING.right;
  const plotHeight = canvas.height - PADDING.top - PADDING.bottom;

  const project = (point: PlotPoint) => ({
    x: PADDING.left + ((point.x - xDomain[0]) / (xDomain[1] - xDomain[0])) * plotWidth,
    y: canvas.height - PADDING.bottom - ((point.y - yDomain[0]) / (yDomain[1] - yDomain[0])) * plotHeight,
  });

  context.clearRect(0, 0, canvas.width, canvas.height);
  context.fillStyle = '#fffdf8';
  context.fillRect(0, 0, canvas.width, canvas.height);

  drawTitle(context, options.title);
  drawGrid(context, canvas, xDomain, yDomain, project);
  drawAxes(context, canvas, xDomain, yDomain, project);

  for (const rectangle of options.rectangles ?? []) {
    const topLeft = project({ x: rectangle.x, y: rectangle.y + rectangle.height });
    const bottomRight = project({ x: rectangle.x + rectangle.width, y: rectangle.y });
    const x = Math.min(topLeft.x, bottomRight.x);
    const y = Math.min(topLeft.y, bottomRight.y);
    const width = Math.abs(bottomRight.x - topLeft.x);
    let height = Math.abs(bottomRight.y - topLeft.y);

    if ((rectangle.anchor ?? 'baseline') === 'baseline') {
      const baseline = project({ x: rectangle.x, y: 0 });
      const valuePoint = project({ x: rectangle.x, y: rectangle.y });
      height = Math.abs(baseline.y - valuePoint.y);
    }

    context.fillStyle = rectangle.fill;
    context.fillRect(x, y, width, height);
    if (rectangle.stroke) {
      context.strokeStyle = rectangle.stroke;
      context.strokeRect(x, y, width, height);
    }
  }

  for (const polygon of options.polygons ?? []) {
    if (polygon.points.length === 0) {
      continue;
    }
    context.beginPath();
    const first = project(polygon.points[0]);
    context.moveTo(first.x, first.y);
    for (const point of polygon.points.slice(1)) {
      const projected = project(point);
      context.lineTo(projected.x, projected.y);
    }
    context.closePath();
    context.fillStyle = polygon.fill;
    context.fill();
    if (polygon.stroke) {
      context.strokeStyle = polygon.stroke;
      context.lineWidth = 1.2;
      context.stroke();
    }
  }

  for (const series of options.lineSeries ?? []) {
    if (series.points.length < 2) {
      continue;
    }
    context.beginPath();
    const first = project(series.points[0]);
    context.moveTo(first.x, first.y);
    for (const point of series.points.slice(1)) {
      const projected = project(point);
      context.lineTo(projected.x, projected.y);
    }
    context.strokeStyle = series.color;
    context.lineWidth = series.width ?? 2.2;
    context.setLineDash(series.dash ?? []);
    context.stroke();
    context.setLineDash([]);
  }

  drawAxisLabels(context, canvas, options.xLabel, options.yLabel);
  drawLegend(context, options.lineSeries ?? []);
}

function drawTitle(context: CanvasRenderingContext2D, title?: string): void {
  if (!title) {
    return;
  }
  context.fillStyle = '#1c1b18';
  context.font = 'bold 15px Georgia';
  context.fillText(title, PADDING.left, 18);
}

function drawAxisLabels(
  context: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement,
  xLabel?: string,
  yLabel?: string,
): void {
  context.fillStyle = 'rgba(28, 27, 24, 0.62)';
  context.font = '12px Georgia';
  if (xLabel) {
    context.fillText(xLabel, canvas.width * 0.5 - 18, canvas.height - 8);
  }
  if (yLabel) {
    context.save();
    context.translate(14, canvas.height * 0.5 + 18);
    context.rotate(-Math.PI / 2);
    context.fillText(yLabel, 0, 0);
    context.restore();
  }
}

function drawLegend(context: CanvasRenderingContext2D, series: LineSeries[]): void {
  const entries = series.filter((item) => item.label);
  if (entries.length === 0) {
    return;
  }

  let x = PADDING.left;
  const y = 34;
  context.font = '12px Georgia';
  for (const entry of entries) {
    context.strokeStyle = entry.color;
    context.lineWidth = 2.4;
    context.beginPath();
    context.moveTo(x, y);
    context.lineTo(x + 18, y);
    context.stroke();
    context.fillStyle = '#1c1b18';
    context.fillText(entry.label ?? '', x + 24, y + 4);
    x += 24 + context.measureText(entry.label ?? '').width + 18;
  }
}

function resolveDomains(options: PlotOptions): { xDomain: [number, number]; yDomain: [number, number] } {
  const xValues: number[] = [];
  const yValues: number[] = [];

  for (const series of options.lineSeries ?? []) {
    for (const point of series.points) {
      xValues.push(point.x);
      yValues.push(point.y);
    }
  }

  for (const polygon of options.polygons ?? []) {
    for (const point of polygon.points) {
      xValues.push(point.x);
      yValues.push(point.y);
    }
  }

  for (const rectangle of options.rectangles ?? []) {
    xValues.push(rectangle.x, rectangle.x + rectangle.width);
    yValues.push(0, rectangle.y, rectangle.y + rectangle.height);
  }

  const xDomain = options.xDomain ?? paddedDomain(xValues.length ? xValues : [0, 1]);
  const yDomain = options.yDomain ?? paddedDomain(yValues.length ? yValues : [-1, 1]);
  return { xDomain, yDomain };
}

function paddedDomain(values: number[]): [number, number] {
  let min = Math.min(...values);
  let max = Math.max(...values);
  if (min === max) {
    const padding = min === 0 ? 1 : Math.abs(min) * 0.2;
    min -= padding;
    max += padding;
  }
  const padding = (max - min) * 0.12;
  return [min - padding, max + padding];
}

function drawGrid(
  context: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement,
  xDomain: [number, number],
  yDomain: [number, number],
  project: (point: PlotPoint) => { x: number; y: number },
): void {
  context.strokeStyle = 'rgba(28, 27, 24, 0.08)';
  context.lineWidth = 1;
  for (let index = 0; index <= 4; index += 1) {
    const x = xDomain[0] + ((xDomain[1] - xDomain[0]) * index) / 4;
    const from = project({ x, y: yDomain[0] });
    const to = project({ x, y: yDomain[1] });
    context.beginPath();
    context.moveTo(from.x, from.y);
    context.lineTo(to.x, to.y);
    context.stroke();

    const y = yDomain[0] + ((yDomain[1] - yDomain[0]) * index) / 4;
    const left = project({ x: xDomain[0], y });
    const right = project({ x: xDomain[1], y });
    context.beginPath();
    context.moveTo(left.x, left.y);
    context.lineTo(right.x, right.y);
    context.stroke();
  }

  context.fillStyle = 'rgba(28, 27, 24, 0.55)';
  context.font = '12px Georgia';
  context.fillText(xDomain[0].toFixed(2), PADDING.left, canvas.height - 10);
  context.fillText(xDomain[1].toFixed(2), canvas.width - PADDING.right - 32, canvas.height - 10);
  context.fillText(yDomain[0].toFixed(2), 6, canvas.height - PADDING.bottom);
  context.fillText(yDomain[1].toFixed(2), 6, PADDING.top + 8);
}

function drawAxes(
  context: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement,
  xDomain: [number, number],
  yDomain: [number, number],
  project: (point: PlotPoint) => { x: number; y: number },
): void {
  context.strokeStyle = 'rgba(28, 27, 24, 0.5)';
  context.lineWidth = 1.2;
  if (yDomain[0] <= 0 && yDomain[1] >= 0) {
    const left = project({ x: xDomain[0], y: 0 });
    const right = project({ x: xDomain[1], y: 0 });
    context.beginPath();
    context.moveTo(left.x, left.y);
    context.lineTo(right.x, right.y);
    context.stroke();
  }
  if (xDomain[0] <= 0 && xDomain[1] >= 0) {
    const top = project({ x: 0, y: yDomain[1] });
    const bottom = project({ x: 0, y: yDomain[0] });
    context.beginPath();
    context.moveTo(top.x, top.y);
    context.lineTo(bottom.x, bottom.y);
    context.stroke();
  }
}
