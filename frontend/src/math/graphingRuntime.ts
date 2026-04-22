import type { LineSeries, PlotPoint, PlotRect } from '../render2d/canvasPlot.ts';
import type { GraphExpressionTranslation, RelationOperator } from './engineExpression.ts';
import type { GraphCurveSnapshot, GraphRelationGridSnapshot } from '../wasm/engine.ts';

type RuntimeToken =
  | { type: 'number'; value: string }
  | { type: 'identifier'; value: string }
  | { type: 'symbol'; value: string }
  | { type: 'end'; value: '' };

type RuntimeExpression =
  | { kind: 'literal'; value: number }
  | { kind: 'variable'; name: 'x' | 'y' }
  | { kind: 'constant'; name: 'pi' | 'e' }
  | { kind: 'unary'; operator: '+' | '-'; operand: RuntimeExpression }
  | {
      kind: 'binary';
      operator: '+' | '-' | '*' | '/' | '^';
      left: RuntimeExpression;
      right: RuntimeExpression;
    }
  | { kind: 'call'; name: string; argument: RuntimeExpression };

export interface GraphViewport {
  xDomain: [number, number];
  yDomain: [number, number];
  resolution: number;
}

export interface GraphRenderableExpression {
  id: string;
  label: string;
  color: string;
  translation: GraphExpressionTranslation;
}

export interface GraphPlotData {
  lineSeries: LineSeries[];
  rectangles: PlotRect[];
  issues: string[];
}

const FUNCTION_NAMES = new Set(['sin', 'cos', 'tan', 'asin', 'acos', 'atan', 'exp', 'ln', 'log', 'sqrt', 'abs']);
const compileCache = new Map<string, RuntimeExpression>();

export function buildGraphPlot(
  entries: GraphRenderableExpression[],
  viewport: GraphViewport,
): GraphPlotData {
  const lineSeries: LineSeries[] = [];
  const rectangles: PlotRect[] = [];
  const issues: string[] = [];

  for (const entry of entries) {
    try {
      if (entry.translation.kind === 'explicit-function') {
        lineSeries.push(
          ...sampleExplicit(entry.label, entry.color, entry.translation.expression, viewport),
        );
        continue;
      }

      if (entry.translation.kind === 'implicit-relation') {
        const relationPlot = plotImplicitRelation(entry, viewport);
        lineSeries.push(...relationPlot.lineSeries);
        rectangles.push(...relationPlot.rectangles);
      }
    } catch (error) {
      issues.push(
        error instanceof Error
          ? `${entry.label}: ${error.message}`
          : `${entry.label}: ${String(error)}`,
      );
    }
  }

  return { lineSeries, rectangles, issues };
}

export function plotRustCurveSamples(
  label: string,
  color: string,
  snapshot: GraphCurveSnapshot,
  yDomain: [number, number],
): LineSeries[] {
  const series: LineSeries[] = [];
  const points: PlotPoint[] = [];
  const ySpan = yDomain[1] - yDomain[0];
  let previousY: number | null = null;

  for (let index = 0; index < snapshot.x.length; index += 1) {
    const x = snapshot.x[index];
    const y = snapshot.y[index];
    const valid = Number.isFinite(x) && Number.isFinite(y);
    const jump = previousY !== null && valid && Math.abs(y - previousY) > ySpan * 1.4;

    if (!valid || jump) {
      flushSeries(series, points, color, label);
      previousY = null;
      continue;
    }

    points.push({ x, y });
    previousY = y;
  }

  flushSeries(series, points, color, label);
  return series;
}

export function plotRustRelationGrid(
  label: string,
  color: string,
  snapshot: GraphRelationGridSnapshot,
): Pick<GraphPlotData, 'lineSeries' | 'rectangles'> {
  const relation = (snapshot.relation ?? '=') as RelationOperator;
  const lineSeries: LineSeries[] = [];
  const rectangles: PlotRect[] = [];
  const resolution = snapshot.resolution;
  const dx = (snapshot.xMax - snapshot.xMin) / resolution;
  const dy = (snapshot.yMax - snapshot.yMin) / resolution;
  let labeledBoundary = false;

  for (let ix = 0; ix < resolution; ix += 1) {
    const x0 = snapshot.xMin + ix * dx;
    const x1 = x0 + dx;
    for (let iy = 0; iy < resolution; iy += 1) {
      const y0 = snapshot.yMin + iy * dy;
      const y1 = y0 + dy;
      const values: [number, number, number, number] = [
        vertexValue(snapshot, ix, iy),
        vertexValue(snapshot, ix + 1, iy),
        vertexValue(snapshot, ix + 1, iy + 1),
        vertexValue(snapshot, ix, iy + 1),
      ];

      if (supportsRegionFill(relation)) {
        const centerValue = cellValue(snapshot, ix, iy);
        if (testRelation(relation, centerValue)) {
          rectangles.push({
            x: x0,
            y: y0,
            width: dx,
            height: dy,
            fill: hexToRgba(color, 0.09),
            anchor: 'corner',
          });
        }
      }

      for (const segment of contourSegments(values, x0, x1, y0, y1)) {
        lineSeries.push({
          color,
          width: relation === '!=' ? 1.6 : 2.1,
          dash: relation === '!=' ? [8, 6] : undefined,
          label: labeledBoundary ? undefined : label,
          points: segment,
        });
        labeledBoundary = true;
      }
    }
  }

  return { lineSeries, rectangles };
}

function sampleExplicit(
  label: string,
  color: string,
  expression: string,
  viewport: GraphViewport,
): LineSeries[] {
  const compiled = compileExpression(expression);
  const series: LineSeries[] = [];
  const points: PlotPoint[] = [];
  const sampleCount = Math.max(320, viewport.resolution * 4);
  const ySpan = viewport.yDomain[1] - viewport.yDomain[0];

  let previousY: number | null = null;
  for (let index = 0; index < sampleCount; index += 1) {
    const ratio = index / (sampleCount - 1);
    const x = viewport.xDomain[0] + (viewport.xDomain[1] - viewport.xDomain[0]) * ratio;
    const y = evaluateExpression(compiled, x, 0);
    const valid = Number.isFinite(y);
    const jump = previousY !== null && valid && Math.abs(y - previousY) > ySpan * 1.4;

    if (!valid || jump) {
      flushSeries(series, points, color, label);
      previousY = null;
      continue;
    }

    points.push({ x, y });
    previousY = y;
  }

  flushSeries(series, points, color, label);
  return series;
}

function plotImplicitRelation(
  entry: GraphRenderableExpression,
  viewport: GraphViewport,
): Pick<GraphPlotData, 'lineSeries' | 'rectangles'> {
  const relation = entry.translation.relation;
  const left = compileExpression(entry.translation.leftExpression);
  const right = compileExpression(entry.translation.rightExpression);
  const lineSeries: LineSeries[] = [];
  const rectangles: PlotRect[] = [];
  const resolution = Math.max(28, viewport.resolution);
  const dx = (viewport.xDomain[1] - viewport.xDomain[0]) / resolution;
  const dy = (viewport.yDomain[1] - viewport.yDomain[0]) / resolution;
  let labeledBoundary = false;

  for (let ix = 0; ix < resolution; ix += 1) {
    const x0 = viewport.xDomain[0] + ix * dx;
    const x1 = x0 + dx;
    for (let iy = 0; iy < resolution; iy += 1) {
      const y0 = viewport.yDomain[0] + iy * dy;
      const y1 = y0 + dy;
      const values = [
        difference(left, right, x0, y0),
        difference(left, right, x1, y0),
        difference(left, right, x1, y1),
        difference(left, right, x0, y1),
      ];

      if (supportsRegionFill(relation)) {
        const centerValue = difference(left, right, x0 + dx * 0.5, y0 + dy * 0.5);
        if (testRelation(relation, centerValue)) {
          rectangles.push({
            x: x0,
            y: y0,
            width: dx,
            height: dy,
            fill: hexToRgba(entry.color, 0.09),
            anchor: 'corner',
          });
        }
      }

      for (const segment of contourSegments(values, x0, x1, y0, y1)) {
        lineSeries.push({
          color: entry.color,
          width: relation === '!=' ? 1.6 : 2.1,
          dash: relation === '!=' ? [8, 6] : undefined,
          label: labeledBoundary ? undefined : entry.label,
          points: segment,
        });
        labeledBoundary = true;
      }
    }
  }

  return { lineSeries, rectangles };
}

function difference(
  left: RuntimeExpression,
  right: RuntimeExpression,
  x: number,
  y: number,
): number {
  return evaluateExpression(left, x, y) - evaluateExpression(right, x, y);
}

function supportsRegionFill(relation: RelationOperator): boolean {
  return relation === '<' || relation === '<=' || relation === '>' || relation === '>=';
}

function testRelation(relation: RelationOperator, value: number): boolean {
  switch (relation) {
    case '<':
      return value < 0;
    case '<=':
      return value <= 0;
    case '>':
      return value > 0;
    case '>=':
      return value >= 0;
    case '!=':
      return Math.abs(value) > 1e-6;
    case '=':
      return Math.abs(value) < 1e-3;
  }
}

function contourSegments(
  values: [number, number, number, number] | number[],
  x0: number,
  x1: number,
  y0: number,
  y1: number,
): PlotPoint[][] {
  const corners: PlotPoint[] = [
    { x: x0, y: y0 },
    { x: x1, y: y0 },
    { x: x1, y: y1 },
    { x: x0, y: y1 },
  ];
  const edges: Array<[number, number]> = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
  ];
  const intersections: PlotPoint[] = [];

  for (const [startIndex, endIndex] of edges) {
    const a = values[startIndex];
    const b = values[endIndex];
    if (!Number.isFinite(a) || !Number.isFinite(b)) {
      continue;
    }
    if ((a > 0 && b > 0) || (a < 0 && b < 0)) {
      continue;
    }
    if (a === b) {
      continue;
    }
    const t = a / (a - b);
    intersections.push({
      x: corners[startIndex].x + (corners[endIndex].x - corners[startIndex].x) * t,
      y: corners[startIndex].y + (corners[endIndex].y - corners[startIndex].y) * t,
    });
  }

  if (intersections.length === 2) {
    return [[intersections[0], intersections[1]]];
  }

  if (intersections.length === 4) {
    const center = (values[0] + values[1] + values[2] + values[3]) * 0.25;
    if (center >= 0) {
      return [
        [intersections[0], intersections[1]],
        [intersections[2], intersections[3]],
      ];
    }
    return [
      [intersections[0], intersections[3]],
      [intersections[1], intersections[2]],
    ];
  }

  return [];
}

function vertexValue(snapshot: GraphRelationGridSnapshot, ix: number, iy: number): number {
  return snapshot.vertexValues[iy * (snapshot.resolution + 1) + ix] ?? Number.NaN;
}

function cellValue(snapshot: GraphRelationGridSnapshot, ix: number, iy: number): number {
  return snapshot.cellValues[iy * snapshot.resolution + ix] ?? Number.NaN;
}

function flushSeries(series: LineSeries[], points: PlotPoint[], color: string, label: string): void {
  if (points.length < 2) {
    points.length = 0;
    return;
  }

  series.push({
    points: points.splice(0, points.length),
    color,
    width: 2.4,
    label: series.length === 0 ? label : undefined,
  });
}

function compileExpression(source: string): RuntimeExpression {
  const cached = compileCache.get(source);
  if (cached) {
    return cached;
  }

  const parser = new RuntimeParser(source);
  const compiled = parser.parse();
  compileCache.set(source, compiled);
  return compiled;
}

function evaluateExpression(expression: RuntimeExpression, x: number, y: number): number {
  switch (expression.kind) {
    case 'literal':
      return expression.value;
    case 'variable':
      return expression.name === 'x' ? x : y;
    case 'constant':
      return expression.name === 'pi' ? Math.PI : Math.E;
    case 'unary': {
      const value = evaluateExpression(expression.operand, x, y);
      return expression.operator === '-' ? -value : value;
    }
    case 'binary': {
      const left = evaluateExpression(expression.left, x, y);
      const right = evaluateExpression(expression.right, x, y);
      switch (expression.operator) {
        case '+':
          return left + right;
        case '-':
          return left - right;
        case '*':
          return left * right;
        case '/':
          return Math.abs(right) < 1e-12 ? Number.NaN : left / right;
        case '^':
          if (left < 0 && Math.abs(right - Math.round(right)) > 1e-10) {
            return Number.NaN;
          }
          return left ** right;
      }
    }
    case 'call': {
      const value = evaluateExpression(expression.argument, x, y);
      switch (expression.name) {
        case 'sin':
          return Math.sin(value);
        case 'cos':
          return Math.cos(value);
        case 'tan':
          return Math.abs(Math.cos(value)) < 1e-12 ? Number.NaN : Math.tan(value);
        case 'asin':
          return value < -1 || value > 1 ? Number.NaN : Math.asin(value);
        case 'acos':
          return value < -1 || value > 1 ? Number.NaN : Math.acos(value);
        case 'atan':
          return Math.atan(value);
        case 'exp':
          return Math.exp(value);
        case 'ln':
          return value <= 0 ? Number.NaN : Math.log(value);
        case 'log':
          return value <= 0 ? Number.NaN : Math.log10(value);
        case 'sqrt':
          return value < 0 ? Number.NaN : Math.sqrt(value);
        case 'abs':
          return Math.abs(value);
        default:
          return Number.NaN;
      }
    }
  }
}

class RuntimeParser {
  private readonly tokens: RuntimeToken[];
  private index = 0;

  constructor(source: string) {
    this.tokens = tokenizeRuntime(source);
  }

  parse(): RuntimeExpression {
    const expression = this.parseAdditive();
    if (this.peek().type !== 'end') {
      throw new Error(`Unexpected trailing input near "${this.peek().value}".`);
    }
    return expression;
  }

  private parseAdditive(): RuntimeExpression {
    let expression = this.parseMultiplicative();
    while (this.match('+') || this.match('-')) {
      const operator = this.previous().value as '+' | '-';
      expression = {
        kind: 'binary',
        operator,
        left: expression,
        right: this.parseMultiplicative(),
      };
    }
    return expression;
  }

  private parseMultiplicative(): RuntimeExpression {
    let expression = this.parsePower();
    while (this.match('*') || this.match('/')) {
      const operator = this.previous().value as '*' | '/';
      expression = {
        kind: 'binary',
        operator,
        left: expression,
        right: this.parsePower(),
      };
    }
    return expression;
  }

  private parsePower(): RuntimeExpression {
    let expression = this.parseUnary();
    if (this.match('^')) {
      expression = {
        kind: 'binary',
        operator: '^',
        left: expression,
        right: this.parsePower(),
      };
    }
    return expression;
  }

  private parseUnary(): RuntimeExpression {
    if (this.match('+')) {
      return { kind: 'unary', operator: '+', operand: this.parseUnary() };
    }
    if (this.match('-')) {
      return { kind: 'unary', operator: '-', operand: this.parseUnary() };
    }
    return this.parsePrimary();
  }

  private parsePrimary(): RuntimeExpression {
    const token = this.peek();
    if (token.type === 'number') {
      this.advance();
      return { kind: 'literal', value: Number(token.value) };
    }
    if (token.type === 'identifier') {
      this.advance();
      if (FUNCTION_NAMES.has(token.value)) {
        this.expect('(');
        const argument = this.parseAdditive();
        this.expect(')');
        return { kind: 'call', name: token.value, argument };
      }
      if (token.value === 'x' || token.value === 'y') {
        return { kind: 'variable', name: token.value };
      }
      if (token.value === 'pi' || token.value === 'e') {
        return { kind: 'constant', name: token.value };
      }
      throw new Error(`Unsupported symbol "${token.value}".`);
    }
    if (this.match('(')) {
      const expression = this.parseAdditive();
      this.expect(')');
      return expression;
    }
    throw new Error(`Expected an expression near "${token.value}".`);
  }

  private match(symbol: string): boolean {
    if (this.peek().type === 'symbol' && this.peek().value === symbol) {
      this.advance();
      return true;
    }
    return false;
  }

  private expect(symbol: string): void {
    if (!this.match(symbol)) {
      throw new Error(`Expected "${symbol}".`);
    }
  }

  private previous(): RuntimeToken {
    return this.tokens[this.index - 1];
  }

  private peek(): RuntimeToken {
    return this.tokens[this.index] ?? { type: 'end', value: '' };
  }

  private advance(): void {
    if (this.index < this.tokens.length - 1) {
      this.index += 1;
    }
  }
}

function tokenizeRuntime(source: string): RuntimeToken[] {
  const tokens: RuntimeToken[] = [];
  for (let index = 0; index < source.length; ) {
    const char = source[index];
    if (/[0-9.]/.test(char)) {
      let end = index + 1;
      while (end < source.length && /[0-9.]/.test(source[end])) {
        end += 1;
      }
      tokens.push({ type: 'number', value: source.slice(index, end) });
      index = end;
      continue;
    }
    if (/[a-zA-Z]/.test(char)) {
      let end = index + 1;
      while (end < source.length && /[a-zA-Z]/.test(source[end])) {
        end += 1;
      }
      tokens.push({ type: 'identifier', value: source.slice(index, end) });
      index = end;
      continue;
    }
    if ('+-*/^()'.includes(char)) {
      tokens.push({ type: 'symbol', value: char });
      index += 1;
      continue;
    }
    throw new Error(`Unsupported runtime character "${char}".`);
  }
  tokens.push({ type: 'end', value: '' });
  return tokens;
}

function hexToRgba(hex: string, alpha: number): string {
  const normalized = hex.replace('#', '');
  const value = normalized.length === 3
    ? normalized.split('').map((part) => `${part}${part}`).join('')
    : normalized;
  const red = Number.parseInt(value.slice(0, 2), 16);
  const green = Number.parseInt(value.slice(2, 4), 16);
  const blue = Number.parseInt(value.slice(4, 6), 16);
  return `rgba(${red}, ${green}, ${blue}, ${alpha})`;
}