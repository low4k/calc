import {
  engineExpressionToLatex,
  INLINE_SHORTCUTS,
  SHORTCUT_GROUPS,
  translateGraphingLatex,
  type GraphExpressionTranslation,
} from '../math/engineExpression.ts';
import { plotRustCurveSamples, plotRustRelationGrid } from '../math/graphingRuntime.ts';
import { createCanvasPlot, renderCartesianPlot } from '../render2d/canvasPlot.ts';
import type {
  GraphCurveSnapshot,
  GraphExpressionSnapshot,
  GraphRelationGridSnapshot,
} from '../wasm/engine.ts';

interface MathFieldLike extends HTMLElement {
  value: string;
  inlineShortcutTimeout: number;
  mathVirtualKeyboardPolicy: 'auto' | 'manual' | 'sandboxed';
  inlineShortcuts: Record<string, unknown>;
  insert(latex: string): boolean;
  focus(): void;
}

interface ExpressionRow {
  id: string;
  color: string;
  container: HTMLElement;
  field: MathFieldLike | null;
  host: HTMLElement;
  visibleInput: HTMLInputElement;
  colorInput: HTMLInputElement;
  label: HTMLElement;
  status: HTMLElement;
  removeButton: HTMLButtonElement;
  latex: string;
  translation: GraphExpressionTranslation | null;
  error: string | null;
  rustSummary: GraphExpressionSnapshot | null;
  rustError: string | null;
  analysisVersion: number;
}

const ROW_COLORS = ['#a33f27', '#2f6f59', '#1f4c8c', '#8c4f1f', '#6a3ea1', '#0d6b74'];
const EXAMPLES = [
  { label: 'sin(x)', latex: 'y=\\sin\\left(x\\right)' },
  { label: '\u222b t dt (area)', latex: 'y=\\int_{0}^{x}t\\,dt' },
  { label: 'Circle', latex: 'x^2+y^2=1' },
  { label: 'Parabola region', latex: 'y>=x^2-1' },
  { label: '\u03a3 1/n', latex: 'y=\\sum_{n=1}^{x}\\frac{1}{n}' },
  { label: '1/x', latex: 'y=\\frac{1}{x}' },
  { label: 'sqrt(x)', latex: 'y=\\sqrt{x}' },
  { label: '\u03a0(1+1/n)', latex: 'y=\\prod_{n=1}^{x}\\left(1+\\frac{1}{n}\\right)' },
];

export function createFunctionInput(
  defaultExpression: string,
  onApply: (expression: string) => Promise<string>,
  onClearAppliedExpression: () => Promise<void>,
  onAnalyzeGraphExpression: (expression: string) => Promise<GraphExpressionSnapshot>,
  onSampleGraphCurve: (
    expression: string,
    request: { start: number; end: number; sampleCount: number },
  ) => Promise<GraphCurveSnapshot>,
  onBuildGraphRelationGrid: (
    expression: string,
    request: { xMin: number; xMax: number; yMin: number; yMax: number; resolution: number },
  ) => Promise<GraphRelationGridSnapshot>,
): HTMLElement {
  const section = document.createElement('section');
  section.className = 'panel panel-wide graphing-panel';
  section.innerHTML = '<h2>Graphing Calculator</h2>';

  const intro = document.createElement('p');
  intro.className = 'panel-copy';
  intro.textContent = 'Type expressions like a graphing calculator. Type “int” for a cumulative integral ∫₀ˣ f(t) dt, “sum” for Σₙ₌₁ˣ f(n), or “prod” for Π f(n). Expressions that depend on x drive the Riemann, Taylor, and revolution panels automatically.';

  const layout = document.createElement('div');
  layout.className = 'graphing-layout';

  const sidebar = document.createElement('aside');
  sidebar.className = 'expression-sidebar';

  const toolbar = document.createElement('div');
  toolbar.className = 'expression-toolbar';
  const addButton = document.createElement('button');
  addButton.type = 'button';
  addButton.textContent = 'Add Expression';
  const loadExamplesButton = document.createElement('button');
  loadExamplesButton.type = 'button';
  loadExamplesButton.className = 'button-secondary';
  loadExamplesButton.textContent = 'Load Examples';
  const resetViewButton = document.createElement('button');
  resetViewButton.type = 'button';
  resetViewButton.className = 'button-secondary';
  resetViewButton.textContent = 'Reset View';
  toolbar.append(addButton, loadExamplesButton, resetViewButton);

  const examples = document.createElement('div');
  examples.className = 'example-strip';
  for (const example of EXAMPLES) {
    const chip = document.createElement('button');
    chip.type = 'button';
    chip.className = 'example-chip';
    chip.textContent = example.label;
    chip.addEventListener('click', () => {
      addExpressionRow(example.latex, true);
    });
    examples.appendChild(chip);
  }

  const list = document.createElement('div');
  list.className = 'expression-list';

  const shortcuts = document.createElement('div');
  shortcuts.className = 'shortcut-groups shortcut-groups-compact';

  const stage = document.createElement('div');
  stage.className = 'graph-stage';

  const backendStatus = document.createElement('p');
  backendStatus.className = 'panel-copy backend-status';
  backendStatus.textContent = 'Type “int”, “sum”, or “prod” to insert a calculus expression, or pick one from the examples above. Explicit functions of x automatically drive the Riemann, Taylor, revolution, and error panels below.';

  const translated = document.createElement('p');
  translated.className = 'panel-copy engine-preview';
  translated.textContent = 'Select or edit an expression to inspect its normalized graph form.';

  const controls = document.createElement('div');
  controls.className = 'control-grid graph-domain-grid';
  const xMinInput = createNumberInput(-6);
  const xMaxInput = createNumberInput(6);
  const yMinInput = createNumberInput(-4);
  const yMaxInput = createNumberInput(4);
  const resolutionInput = createNumberInput(84, 24);
  controls.append(
    wrapControl('x min', xMinInput),
    wrapControl('x max', xMaxInput),
    wrapControl('y min', yMinInput),
    wrapControl('y max', yMaxInput),
    wrapControl('Grid resolution', resolutionInput),
  );

  const canvas = createCanvasPlot();
  const frame = document.createElement('div');
  frame.className = 'plot-frame graph-frame';
  frame.appendChild(canvas);

  const viewportHint = document.createElement('p');
  viewportHint.className = 'panel-copy viewport-hint';
  viewportHint.textContent = 'Drag to pan. Use the mouse wheel or trackpad scroll over the graph to zoom toward the pointer.';

  const result = document.createElement('p');
  result.className = 'panel-copy panel-result';
  result.textContent = 'The graph updates as soon as Rust confirms each expression.';

  sidebar.append(toolbar, examples, list, shortcuts);
  stage.append(backendStatus, translated, controls, frame, viewportHint, result);
  layout.append(sidebar, stage);
  section.append(intro, layout);

  let rowCounter = 0;
  let activeRowId: string | null = null;
  let syncedBackendExpression: string | null = null;
  let syncVersion = 0;
  let renderVersion = 0;
  let mathliveReady: Promise<void> | null = null;
  const rows = new Map<string, ExpressionRow>();
  const viewport = {
    xMin: -6,
    xMax: 6,
    yMin: -4,
    yMax: 4,
    resolution: 84,
  };

  let dragState:
    | {
        pointerId: number;
        startX: number;
        startY: number;
        xMin: number;
        xMax: number;
        yMin: number;
        yMax: number;
      }
    | null = null;

  const ensureMathlive = async () => {
    if (!mathliveReady) {
      mathliveReady = import('mathlive').then(() => undefined);
    }
    return mathliveReady;
  };

  const activeRow = () => (activeRowId ? rows.get(activeRowId) ?? null : null);

  const setActiveRow = (rowId: string) => {
    activeRowId = rowId;
    for (const row of rows.values()) {
      row.container.classList.toggle('expression-row-active', row.id === rowId);
    }
    updateInspector();
    void syncBackendSelection();
  };

  const updateInspector = () => {
    const row = activeRow();
    if (!row) {
      translated.textContent = 'Select an expression to inspect its normalized graph form.';
      translated.className = 'panel-copy engine-preview';
      return;
    }

    if (row.error) {
      translated.textContent = row.error;
      translated.className = 'panel-copy engine-preview engine-preview-error';
      return;
    }

    if (row.rustError) {
      translated.textContent = `Rust graph parser: ${row.rustError}`;
      translated.className = 'panel-copy engine-preview engine-preview-error';
      return;
    }

    if (!row.translation) {
      translated.textContent = 'Loading expression editor...';
      translated.className = 'panel-copy engine-preview';
      return;
    }

    if (row.rustSummary) {
      translated.textContent = row.rustSummary.kind === 'implicit-relation'
        ? `Rust relation: ${row.rustSummary.display}`
        : `Rust normalized: ${row.rustSummary.display}`;
    } else {
      translated.textContent = row.translation.kind === 'explicit-function'
        ? `Normalized: ${row.translation.display}`
        : `Relation: ${row.translation.display}`;
    }
    translated.className = 'panel-copy engine-preview engine-preview-ok';
  };

  const rowGraphExpression = (row: ExpressionRow) => row.rustSummary?.display ?? row.translation?.display ?? null;

  const rowPlotMode = (row: ExpressionRow): 'explicit' | 'implicit' | null => {
    if (!row.translation) {
      return null;
    }

    if (row.rustSummary?.kind === 'implicit-relation') {
      return 'implicit';
    }

    if (row.rustSummary?.kind === 'explicit-function' || row.rustSummary?.kind === 'scalar') {
      return 'explicit';
    }

    return row.translation.kind === 'implicit-relation' ? 'implicit' : 'explicit';
  };

  const renderGraph = async () => {
    const version = ++renderVersion;
    syncViewportInputs();
    const xDomain: [number, number] = [viewport.xMin, viewport.xMax];
    const yDomain: [number, number] = [viewport.yMin, viewport.yMax];
    const resolution = viewport.resolution;

    const visibleRows = Array.from(rows.values()).filter((row) => row.visibleInput.checked);
    const translatedRows = visibleRows.filter((row) => row.translation);

    const plot = {
      lineSeries: [] as Array<ReturnType<typeof plotRustCurveSamples>[number]>,
      rectangles: [] as ReturnType<typeof plotRustRelationGrid>['rectangles'],
      issues: [] as string[],
    };

    for (const row of translatedRows) {
      const expression = rowGraphExpression(row);
      const plotMode = rowPlotMode(row);

      if (!expression || !plotMode) {
        if (!row.rustError) {
          plot.issues.push(`${row.label.textContent ?? row.id}: waiting for Rust graph analysis`);
        }
        continue;
      }

      try {
        if (plotMode === 'explicit') {
          const snapshot = await onSampleGraphCurve(expression, {
            start: xDomain[0],
            end: xDomain[1],
            sampleCount: Math.max(320, resolution * 4),
          });
          if (version !== renderVersion) {
            return;
          }
          plot.lineSeries.push(
            ...plotRustCurveSamples(row.label.textContent ?? row.id, row.color, snapshot, yDomain),
          );
        } else {
          const snapshot = await onBuildGraphRelationGrid(expression, {
            xMin: xDomain[0],
            xMax: xDomain[1],
            yMin: yDomain[0],
            yMax: yDomain[1],
            resolution: Math.max(28, resolution),
          });
          if (version !== renderVersion) {
            return;
          }
          const relationPlot = plotRustRelationGrid(row.label.textContent ?? row.id, row.color, snapshot);
          plot.lineSeries.push(...relationPlot.lineSeries);
          plot.rectangles.push(...relationPlot.rectangles);
        }
      } catch (error) {
        plot.issues.push(
          error instanceof Error
            ? `${row.label.textContent ?? row.id}: ${error.message}`
            : `${row.label.textContent ?? row.id}: ${String(error)}`,
        );
      }
    }

    renderCartesianPlot(canvas, {
      title: 'Live Graph',
      xLabel: 'x',
      yLabel: 'y',
      xDomain,
      yDomain,
      lineSeries: plot.lineSeries,
      rectangles: plot.rectangles,
    });

    const renderedCount = translatedRows.filter((row) => {
      const expression = rowGraphExpression(row);
      return Boolean(expression && rowPlotMode(row));
    }).length;
    const invalidCount = Array.from(rows.values()).filter((row) => row.error || row.rustError).length;
    const pendingCount = translatedRows.filter((row) => !row.error && !row.rustError && !row.rustSummary).length;
    const issueText = plot.issues.length ? ` Notes: ${plot.issues.join(' | ')}` : '';
    result.textContent = `${renderedCount} Rust-rendered graph expression${renderedCount === 1 ? '' : 's'} shown. ${pendingCount} pending. ${invalidCount} invalid.${issueText}`;
  };

  const syncViewportInputs = () => {
    xMinInput.value = formatNumber(viewport.xMin);
    xMaxInput.value = formatNumber(viewport.xMax);
    yMinInput.value = formatNumber(viewport.yMin);
    yMaxInput.value = formatNumber(viewport.yMax);
    resolutionInput.value = String(viewport.resolution);
  };

  const updateViewportFromInputs = () => {
    viewport.xMin = Number(xMinInput.value);
    viewport.xMax = Number(xMaxInput.value);
    viewport.yMin = Number(yMinInput.value);
    viewport.yMax = Number(yMaxInput.value);
    viewport.resolution = Math.max(24, Number(resolutionInput.value));
    if (viewport.xMin === viewport.xMax) {
      viewport.xMax = viewport.xMin + 1;
    }
    if (viewport.yMin === viewport.yMax) {
      viewport.yMax = viewport.yMin + 1;
    }
    void renderGraph();
  };

  const resetViewport = () => {
    viewport.xMin = -6;
    viewport.xMax = 6;
    viewport.yMin = -4;
    viewport.yMax = 4;
    viewport.resolution = 84;
    void renderGraph();
  };

  const rowGraphKind = (row: ExpressionRow) => row.rustSummary?.kind ?? row.translation?.kind ?? null;

  const rowGraphDisplay = (row: ExpressionRow) => row.rustSummary?.display ?? row.translation?.display ?? row.id;

  const rowBackendExpression = (row: ExpressionRow) => {
    if (row.rustSummary?.backendEligible && row.rustSummary.backendExpression) {
      return row.rustSummary.backendExpression;
    }
    if (row.translation?.kind === 'explicit-function' && row.translation.backendEligible) {
      return row.translation.backendExpression;
    }
    return null;
  };

  const isRowBackendReady = (row: ExpressionRow) => {
    if (row.rustSummary) {
      return row.rustSummary.backendEligible && Boolean(row.rustSummary.backendExpression);
    }
    return Boolean(
      row.translation?.kind === 'explicit-function'
        && row.translation.backendEligible
        && row.translation.backendExpression,
    );
  };

  const syncBackendSelection = async () => {
    const version = ++syncVersion;
    const preferred = activeRow();
    const fallback = Array.from(rows.values()).find(
      (row) => isRowBackendReady(row),
    ) ?? null;
    const selected = preferred && isRowBackendReady(preferred)
      ? preferred
      : fallback;

    if (!selected) {
      syncedBackendExpression = null;
      await onClearAppliedExpression();
      backendStatus.textContent = 'Choose an explicit curve such as y = sin(x) or y = int(t,0,x,t) to drive the calculus panels.';
      return;
    }

    const expression = rowBackendExpression(selected);
    const display = rowGraphDisplay(selected);
    if (!expression) {
      syncedBackendExpression = null;
      await onClearAppliedExpression();
      backendStatus.textContent = 'Choose an explicit curve such as y = sin(x) or y = int(t,0,x,t) to drive the calculus panels.';
      return;
    }

    backendStatus.textContent = `Rust calculus source: ${display}`;

    if (expression === syncedBackendExpression) {
      return;
    }

    try {
      await onApply(expression);
      if (version !== syncVersion) {
        return;
      }
      syncedBackendExpression = expression;
      backendStatus.textContent = `Rust calculus source: ${display}`;
    } catch (error) {
      if (version !== syncVersion) {
        return;
      }
      backendStatus.textContent = error instanceof Error ? error.message : String(error);
    }
  };

  const applyRowStatus = (row: ExpressionRow) => {
    if (row.error) {
      row.status.textContent = row.error;
      row.status.className = 'expression-row-status expression-row-status-error';
      return;
    }

    if (!row.translation) {
      row.status.textContent = 'Loading editor...';
      row.status.className = 'expression-row-status';
      return;
    }

    if (row.rustError) {
      row.status.textContent = `Rust graph parser: ${row.rustError}`;
      row.status.className = 'expression-row-status expression-row-status-error';
      return;
    }

    if (!row.rustSummary) {
      row.status.textContent = 'Valid locally. Checking Rust graph parser...';
      row.status.className = 'expression-row-status';
      return;
    }

    if (row.rustSummary.kind === 'explicit-function') {
      row.status.textContent = row.rustSummary.backendEligible && row.rustSummary.backendExpression
        ? `Rust parsed. Backend-ready: ${row.rustSummary.backendExpression}`
        : `Rust parsed. ${row.rustSummary.warning ?? 'Not yet backend-ready.'}`;
      row.status.className = row.rustSummary.backendEligible
        ? 'expression-row-status expression-row-status-ok'
        : 'expression-row-status expression-row-status-local';
      return;
    }

    if (row.rustSummary.kind === 'scalar') {
      const label = row.rustSummary.backendEligible
        ? `Calculus-ready: ${row.rustSummary.backendExpression ?? row.rustSummary.display}`
        : row.rustSummary.warning ?? `Parsed: ${row.rustSummary.display}`;
      row.status.textContent = label;
      row.status.className = row.rustSummary.backendEligible
        ? 'expression-row-status expression-row-status-ok'
        : 'expression-row-status expression-row-status-local';
      return;
    }

    row.status.textContent = row.rustSummary.warning ?? `Rust parsed ${row.rustSummary.relation ?? 'relation'} locally.`;
    row.status.className = 'expression-row-status expression-row-status-local';
  };

  const analyzeRowWithRust = async (row: ExpressionRow, expression: string, version: number) => {
    try {
      const summary = await onAnalyzeGraphExpression(expression);
      if (version !== row.analysisVersion || row.error) {
        return;
      }
      row.rustSummary = summary;
      row.rustError = null;
    } catch (error) {
      if (version !== row.analysisVersion || row.error) {
        return;
      }
      row.rustSummary = null;
      row.rustError = error instanceof Error ? error.message : String(error);
    }

    applyRowStatus(row);
    updateInspector();
    void syncBackendSelection();
  };

  const refreshRowState = (row: ExpressionRow) => {
    if (!row.field) {
      row.translation = null;
      row.error = null;
      row.rustSummary = null;
      row.rustError = null;
      applyRowStatus(row);
      void renderGraph();
      updateInspector();
      return;
    }

    row.latex = row.field.value;
    const parsed = translateGraphingLatex(row.latex);
    row.translation = parsed.translation;
    row.error = parsed.error;

    if (row.error || !row.translation) {
      row.rustSummary = null;
      row.rustError = null;
      row.analysisVersion += 1;
    } else {
      row.rustSummary = null;
      row.rustError = null;
      row.analysisVersion += 1;
      void analyzeRowWithRust(row, row.translation.display, row.analysisVersion);
    }

    applyRowStatus(row);
    updateInspector();
    void renderGraph();
    void syncBackendSelection();
  };

  const mountMathField = async (row: ExpressionRow) => {
    await ensureMathlive();
    const field = document.createElement('math-field') as MathFieldLike;
    field.className = 'math-input expression-math-field';
    field.setAttribute('smart-fence', 'on');
    field.setAttribute('smart-mode', 'on');
    field.setAttribute('smart-superscript', 'on');
    row.host.replaceWith(field);
    field.addEventListener('focusin', () => {
      setActiveRow(row.id);
    });
    field.addEventListener('input', () => {
      refreshRowState(row);
    });
    field.addEventListener('change', () => {
      refreshRowState(row);
    });
    row.field = field;
    field.inlineShortcutTimeout = 750;
    field.mathVirtualKeyboardPolicy = 'manual';
    field.inlineShortcuts = { ...INLINE_SHORTCUTS };
    field.value = row.latex;
    refreshRowState(row);
  };

  const createShortcutPalette = () => {
    shortcuts.replaceChildren();
    for (const group of SHORTCUT_GROUPS) {
      const groupElement = document.createElement('section');
      groupElement.className = 'shortcut-group';
      const title = document.createElement('h3');
      title.textContent = group.title;
      const grid = document.createElement('div');
      grid.className = 'shortcut-grid';
      for (const shortcut of group.shortcuts) {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = shortcut.engineReady ? 'shortcut-chip' : 'shortcut-chip shortcut-chip-muted';
        button.innerHTML = `<strong>${shortcut.typed}</strong><span>${shortcut.label}</span><em>${shortcut.note}</em>`;
        button.addEventListener('click', () => {
          const row = activeRow();
          if (!row?.field) {
            return;
          }
          row.field.insert(shortcut.latex);
          row.field.focus();
          refreshRowState(row);
        });
        grid.appendChild(button);
      }
      groupElement.append(title, grid);
      shortcuts.appendChild(groupElement);
    }
  };

  const removeRow = (rowId: string) => {
    if (rows.size === 1) {
      return;
    }
    const row = rows.get(rowId);
    if (!row) {
      return;
    }
    row.container.remove();
    rows.delete(rowId);
    if (activeRowId === rowId) {
      const nextRow = Array.from(rows.values())[0] ?? null;
      if (nextRow) {
        setActiveRow(nextRow.id);
      } else {
        activeRowId = null;
      }
    }
    void renderGraph();
    void syncBackendSelection();
  };

  const addExpressionRow = (latex: string, activate: boolean) => {
    const id = `expr-${rowCounter + 1}`;
    const row: ExpressionRow = {
      id,
      color: ROW_COLORS[rowCounter % ROW_COLORS.length],
      container: document.createElement('article'),
      field: null,
      host: document.createElement('div'),
      visibleInput: document.createElement('input'),
      colorInput: document.createElement('input'),
      label: document.createElement('strong'),
      status: document.createElement('p'),
      removeButton: document.createElement('button'),
      latex,
      translation: null,
      error: null,
      rustSummary: null,
      rustError: null,
      analysisVersion: 0,
    };
    rowCounter += 1;

    row.container.className = 'expression-row';
    row.container.style.setProperty('--row-color', row.color);
    row.container.addEventListener('click', () => {
      setActiveRow(row.id);
    });

    const head = document.createElement('div');
    head.className = 'expression-row-head';
    const swatch = document.createElement('span');
    swatch.className = 'expression-swatch';
    row.colorInput.type = 'color';
    row.colorInput.className = 'expression-color';
    row.colorInput.value = row.color;
    row.colorInput.addEventListener('click', (event) => {
      event.stopPropagation();
    });
    row.colorInput.addEventListener('input', (event) => {
      event.stopPropagation();
      row.color = row.colorInput.value;
      row.container.style.setProperty('--row-color', row.color);
      void renderGraph();
    });
    row.visibleInput.type = 'checkbox';
    row.visibleInput.checked = true;
    row.visibleInput.addEventListener('change', () => {
      void renderGraph();
      void syncBackendSelection();
    });
    row.label.textContent = `Expression ${rowCounter}`;
    row.removeButton.type = 'button';
    row.removeButton.className = 'expression-remove';
    row.removeButton.textContent = 'Remove';
    row.removeButton.addEventListener('click', (event) => {
      event.stopPropagation();
      removeRow(row.id);
    });
    head.append(swatch, row.visibleInput, row.label, row.colorInput, row.removeButton);

    row.host.className = 'math-input math-input-loading expression-row-loading';
    row.host.textContent = 'Loading visual math input...';

    row.status.className = 'expression-row-status';
    row.status.textContent = 'Loading editor...';

    row.container.append(head, row.host, row.status);
    list.appendChild(row.container);
    rows.set(row.id, row);
    void mountMathField(row);

    if (activate || rows.size === 1) {
      setActiveRow(row.id);
    }
  };

  addButton.addEventListener('click', () => {
    addExpressionRow('y=x', true);
  });

  loadExamplesButton.addEventListener('click', () => {
    for (const row of Array.from(rows.values())) {
      row.container.remove();
    }
    rows.clear();
    activeRowId = null;
    syncedBackendExpression = null;
    rowCounter = 0;
    for (const example of EXAMPLES) {
      addExpressionRow(example.latex, rows.size === 0);
    }
  });

  resetViewButton.addEventListener('click', () => {
    resetViewport();
  });

  xMinInput.addEventListener('input', updateViewportFromInputs);
  xMaxInput.addEventListener('input', updateViewportFromInputs);
  yMinInput.addEventListener('input', updateViewportFromInputs);
  yMaxInput.addEventListener('input', updateViewportFromInputs);
  resolutionInput.addEventListener('input', updateViewportFromInputs);

  frame.addEventListener('pointerdown', (event) => {
    dragState = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      xMin: viewport.xMin,
      xMax: viewport.xMax,
      yMin: viewport.yMin,
      yMax: viewport.yMax,
    };
    frame.setPointerCapture(event.pointerId);
  });

  frame.addEventListener('pointermove', (event) => {
    if (!dragState || dragState.pointerId !== event.pointerId) {
      return;
    }

    const rect = canvas.getBoundingClientRect();
    const dx = event.clientX - dragState.startX;
    const dy = event.clientY - dragState.startY;
    const xSpan = dragState.xMax - dragState.xMin;
    const ySpan = dragState.yMax - dragState.yMin;
    viewport.xMin = dragState.xMin - (dx / rect.width) * xSpan;
    viewport.xMax = dragState.xMax - (dx / rect.width) * xSpan;
    viewport.yMin = dragState.yMin + (dy / rect.height) * ySpan;
    viewport.yMax = dragState.yMax + (dy / rect.height) * ySpan;
    void renderGraph();
  });

  frame.addEventListener('pointerup', (event) => {
    if (dragState?.pointerId === event.pointerId) {
      frame.releasePointerCapture(event.pointerId);
      dragState = null;
    }
  });

  frame.addEventListener('pointercancel', (event) => {
    if (dragState?.pointerId === event.pointerId) {
      dragState = null;
    }
  });

  frame.addEventListener('wheel', (event) => {
    event.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const px = (event.clientX - rect.left) / rect.width;
    const py = (event.clientY - rect.top) / rect.height;
    const xSpan = viewport.xMax - viewport.xMin;
    const ySpan = viewport.yMax - viewport.yMin;
    const focusX = viewport.xMin + xSpan * px;
    const focusY = viewport.yMax - ySpan * py;
    const factor = Math.exp(event.deltaY * 0.0012);
    const nextXSpan = Math.min(80, Math.max(0.05, xSpan * factor));
    const nextYSpan = Math.min(80, Math.max(0.05, ySpan * factor));
    viewport.xMin = focusX - nextXSpan * px;
    viewport.xMax = viewport.xMin + nextXSpan;
    viewport.yMax = focusY + nextYSpan * py;
    viewport.yMin = viewport.yMax - nextYSpan;
    void renderGraph();
  });

  createShortcutPalette();
  addExpressionRow(engineExpressionToLatex(defaultExpression), true);
  syncViewportInputs();
  return section;
}

function createNumberInput(value: number, min?: number): HTMLInputElement {
  const input = document.createElement('input');
  input.type = 'number';
  input.value = String(value);
  if (min !== undefined) {
    input.min = String(min);
  }
  return input;
}

function wrapControl(text: string, control: HTMLElement): HTMLLabelElement {
  const label = document.createElement('label');
  const span = document.createElement('span');
  span.textContent = text;
  label.append(span, control);
  return label;
}

function formatNumber(value: number): string {
  return Number(value.toFixed(4)).toString();
}
