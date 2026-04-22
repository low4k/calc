import type { InlineShortcutDefinitions } from 'mathlive';

const ENGINE_FUNCTIONS = new Set([
  'sin',
  'cos',
  'tan',
  'asin',
  'acos',
  'atan',
  'exp',
  'ln',
  'log',
  'sqrt',
  'abs',
]);

const BOUND_FUNCTIONS = new Set(['int', 'sum', 'prod']);

const TEXT_FUNCTION_ALIASES = new Map<string, string>([
  ['arcsin', 'asin'],
  ['arccos', 'acos'],
  ['arctan', 'atan'],
]);

const UNSUPPORTED_COMMAND_MESSAGES = new Map<string, string>([
  ['lim', 'Limit notation can be entered visually, but limit evaluation is not implemented yet.'],
  ['to', 'Arrow notation can be entered visually, but mapping notation is not graphable yet.'],
]);

export type RelationOperator = '=' | '<' | '<=' | '>' | '>=' | '!=';

export interface MathShortcut {
  typed: string;
  label: string;
  latex: string;
  note: string;
  engineReady: boolean;
}

export interface ShortcutGroup {
  title: string;
  shortcuts: MathShortcut[];
}

export interface TranslationResult {
  expression: string | null;
  error: string | null;
}

export type GraphExpressionTranslation =
  | {
      kind: 'explicit-function';
      expression: string;
      display: string;
      backendEligible: boolean;
      backendExpression: string;
      warning: string | null;
    }
  | {
      kind: 'implicit-relation';
      relation: RelationOperator;
      leftExpression: string;
      rightExpression: string;
      display: string;
      backendEligible: false;
      backendExpression: null;
      warning: string | null;
    };

export interface GraphTranslationResult {
  translation: GraphExpressionTranslation | null;
  error: string | null;
}

type Token =
  | { type: 'number'; value: string }
  | { type: 'identifier'; value: string }
  | { type: 'command'; value: string }
  | { type: 'symbol'; value: string }
  | { type: 'end'; value: '' };

interface ParserOptions {
  allowYVariable: boolean;
}

interface RelationSplit {
  operator: RelationOperator;
  left: string;
  right: string;
}

export const INLINE_SHORTCUTS: InlineShortcutDefinitions = {
  sqrt: '\\sqrt{#?}',
  root: '\\sqrt[#?]{#?}',
  frac: '\\frac{#?}{#?}',
  pi: '\\pi',
  theta: '\\theta',
  alpha: '\\alpha',
  beta: '\\beta',
  gamma: '\\gamma',
  delta: '\\delta',
  // Integral default: cumulative area from 0 to x, integrating over t.
  // This makes the result a function of x so it graphs immediately.
  int: '\\int_{0}^{x}{#?}\\,dt',
  // Sum/product default: indexed from n=1 to x so the result is a function of x.
  sum: '\\sum_{n=1}^{x}{#?}',
  prod: '\\prod_{n=1}^{x}{#?}',
  abs: '\\operatorname{abs}\\left(#?\\right)',
  sin: '\\sin\\left(#?\\right)',
  cos: '\\cos\\left(#?\\right)',
  tan: '\\tan\\left(#?\\right)',
  ln: '\\ln\\left(#?\\right)',
  log: '\\log\\left(#?\\right)',
  exp: '\\exp\\left(#?\\right)',
  oo: '\\infty',
  inf: '\\infty',
  '<=': '\\le',
  '>=': '\\ge',
  '!=': '\\ne',
  '->': '\\to',
};

export const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: 'Core Graphing',
    shortcuts: [
      { typed: 'sqrt', label: 'Square root', latex: '\\sqrt{#?}', note: 'Graphable', engineReady: true },
      { typed: 'frac', label: 'Fraction', latex: '\\frac{#?}{#?}', note: 'Graphable', engineReady: true },
      { typed: 'pi', label: 'Pi', latex: '\\pi', note: 'Graphable', engineReady: true },
      { typed: 'abs', label: 'Absolute value', latex: '\\operatorname{abs}\\left(#?\\right)', note: 'Graphable', engineReady: true },
      { typed: 'sin', label: 'Sine', latex: '\\sin\\left(#?\\right)', note: 'Graphable', engineReady: true },
      { typed: 'cos', label: 'Cosine', latex: '\\cos\\left(#?\\right)', note: 'Graphable', engineReady: true },
      { typed: 'tan', label: 'Tangent', latex: '\\tan\\left(#?\\right)', note: 'Graphable', engineReady: true },
      { typed: 'ln', label: 'Natural log', latex: '\\ln\\left(#?\\right)', note: 'Graphable', engineReady: true },
      { typed: 'log', label: 'Base-10 log', latex: '\\log\\left(#?\\right)', note: 'Graphable', engineReady: true },
      { typed: 'exp', label: 'Exponential', latex: '\\exp\\left(#?\\right)', note: 'Graphable', engineReady: true },
    ],
  },
  {
    title: 'Relations And Calculus',
    shortcuts: [
      { typed: 'int', label: 'Integral ∫₀ˣ f(t) dt', latex: '\\int_{0}^{x}{#?}\\,dt', note: 'Rust graph + panels', engineReady: true },
      { typed: 'sum', label: 'Sum Σₙ₌₁ˣ f(n)', latex: '\\sum_{n=1}^{x}{#?}', note: 'Rust graph + panels', engineReady: true },
      { typed: 'prod', label: 'Product Πₙ₌₁ˣ f(n)', latex: '\\prod_{n=1}^{x}{#?}', note: 'Rust graph + panels', engineReady: true },
      { typed: 'root', label: 'n-th root', latex: '\\sqrt[#?]{#?}', note: 'Graphable', engineReady: true },
      { typed: 'theta', label: 'Theta', latex: '\\theta', note: 'Symbol only', engineReady: false },
      { typed: 'alpha', label: 'Alpha', latex: '\\alpha', note: 'Symbol only', engineReady: false },
      { typed: 'beta', label: 'Beta', latex: '\\beta', note: 'Symbol only', engineReady: false },
      { typed: 'gamma', label: 'Gamma', latex: '\\gamma', note: 'Symbol only', engineReady: false },
      { typed: 'oo', label: 'Infinity', latex: '\\infty', note: 'Symbol only', engineReady: false },
      { typed: '<=', label: 'Less or equal', latex: '\\le', note: 'Graphable region', engineReady: true },
      { typed: '>=', label: 'Greater or equal', latex: '\\ge', note: 'Graphable region', engineReady: true },
      { typed: '!=', label: 'Not equal', latex: '\\ne', note: 'Boundary only', engineReady: true },
    ],
  },
];

export function engineExpressionToLatex(expression: string): string {
  return expression
    .replace(/pi/g, '\\pi')
    .replace(/asin\(/g, '\\asin\\left(')
    .replace(/acos\(/g, '\\acos\\left(')
    .replace(/atan\(/g, '\\atan\\left(')
    .replace(/sin\(/g, '\\sin\\left(')
    .replace(/cos\(/g, '\\cos\\left(')
    .replace(/tan\(/g, '\\tan\\left(')
    .replace(/ln\(/g, '\\ln\\left(')
    .replace(/log\(/g, '\\log\\left(')
    .replace(/exp\(/g, '\\exp\\left(')
    .replace(/sqrt\(/g, '\\sqrt{')
    .replace(/abs\(/g, '\\operatorname{abs}\\left(')
    .replace(/\)/g, '\\right)')
    .replace(/\\sqrt\{([^}]*)\\right\)/g, '\\sqrt{$1}')
    .replace(/\\operatorname\{abs\}\\left\(([^)]*)\\right\)/g, '\\operatorname{abs}\\left($1\\right)');
}

export function translateMathFieldLatex(latex: string): TranslationResult {
  const graphResult = translateGraphingLatex(latex);
  if (graphResult.error) {
    return { expression: null, error: graphResult.error };
  }

  const translation = graphResult.translation;
  if (!translation || translation.kind !== 'explicit-function') {
    return {
      expression: null,
      error: 'Only explicit y = f(x) expressions can be sent into the Rust calculus panels right now.',
    };
  }

  if (containsVariable(translation.expression, 'y')) {
    return {
      expression: null,
      error: 'The Rust calculus panels still expect functions of x only.',
    };
  }

  return { expression: translation.expression, error: null };
}

export function translateGraphingLatex(latex: string): GraphTranslationResult {
  const compact = sanitizeLatex(latex);
  if (!compact) {
    return { translation: null, error: 'Enter an expression to graph.' };
  }

  if (compact.includes('#?') || compact.includes('\\placeholder')) {
    return { translation: null, error: 'Fill all placeholders before graphing the expression.' };
  }

  try {
    const relation = findTopLevelRelation(compact);
    if (relation) {
      const leftExpression = parseLatexExpression(relation.left, { allowYVariable: true });
      const rightExpression = parseLatexExpression(relation.right, { allowYVariable: true });
      const display = `${leftExpression} ${relation.operator} ${rightExpression}`;

      if (relation.operator === '=' && leftExpression === 'y') {
        return {
          translation: {
            kind: 'explicit-function',
            expression: rightExpression,
            display,
            backendEligible: !containsVariable(rightExpression, 'y'),
            backendExpression: rightExpression,
            warning: containsVariable(rightExpression, 'y')
              ? 'This expression uses y on the right-hand side, so the Rust calculus panels cannot consume it yet.'
              : null,
          },
          error: null,
        };
      }

      if (relation.operator === '=' && rightExpression === 'y') {
        return {
          translation: {
            kind: 'explicit-function',
            expression: leftExpression,
            display: `y = ${leftExpression}`,
            backendEligible: !containsVariable(leftExpression, 'y'),
            backendExpression: leftExpression,
            warning: containsVariable(leftExpression, 'y')
              ? 'This expression uses y on the right-hand side, so the Rust calculus panels cannot consume it yet.'
              : null,
          },
          error: null,
        };
      }

      return {
        translation: {
          kind: 'implicit-relation',
          relation: relation.operator,
          leftExpression,
          rightExpression,
          display,
          backendEligible: false,
          backendExpression: null,
          warning:
            relation.operator === '!='
              ? 'Not-equal relations render the boundary only for now.'
              : null,
        },
        error: null,
      };
    }

    const stripped = stripAssignablePrefix(compact);
    const expression = parseLatexExpression(stripped, { allowYVariable: true });
    if (containsVariable(expression, 'y')) {
      return {
        translation: null,
        error: 'A lone expression containing y needs a relation, such as y = x^2, x^2 + y^2 = 1, or y >= x.',
      };
    }

    return {
      translation: {
        kind: 'explicit-function',
        expression,
        display: `y = ${expression}`,
        backendEligible: true,
        backendExpression: expression,
        warning: null,
      },
      error: null,
    };
  } catch (error) {
    return {
      translation: null,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function sanitizeLatex(latex: string): string {
  // Replace spacing commands with a space so adjacent identifiers keep their
  // word boundary (e.g. "t\,dt" becomes "t dt", not "tdt").
  return latex
    .replace(/\\left/g, '')
    .replace(/\\right/g, '')
    .replace(/\\!/g, ' ')
    .replace(/\\,/g, ' ')
    .replace(/\\:/g, ' ')
    .replace(/\\;/g, ' ')
    .replace(/\\quad/g, ' ')
    .replace(/\\qquad/g, ' ');
  // Note: whitespace is NOT collapsed here — the tokenizer skips it, which
  // lets "t dt" tokenize as two separate identifiers instead of "tdt".
}

function stripAssignablePrefix(source: string): string {
  if (source.startsWith('y=')) {
    return source.slice(2);
  }

  const functionMatch = source.match(/^[a-zA-Z]\?\(x\)=/);
  if (functionMatch) {
    return source.slice(functionMatch[0].length);
  }

  return source;
}

function containsVariable(expression: string, variable: 'x' | 'y'): boolean {
  return new RegExp(`(^|[^a-zA-Z])${variable}([^a-zA-Z]|$)`).test(expression);
}

function parseLatexExpression(source: string, options: ParserOptions): string {
  const parser = new LatexSubsetParser(source, options);
  return trimOuterParens(parser.parse());
}

function trimOuterParens(expression: string): string {
  let result = expression;
  while (result.startsWith('(') && result.endsWith(')')) {
    let depth = 0;
    let balanced = true;
    for (let index = 0; index < result.length; index += 1) {
      const char = result[index];
      if (char === '(') {
        depth += 1;
      } else if (char === ')') {
        depth -= 1;
        if (depth === 0 && index < result.length - 1) {
          balanced = false;
          break;
        }
      }
    }
    if (!balanced || depth !== 0) {
      break;
    }
    result = result.slice(1, -1);
  }
  return result;
}

function findTopLevelRelation(source: string): RelationSplit | null {
  let parenDepth = 0;
  let braceDepth = 0;
  let bracketDepth = 0;

  for (let index = 0; index < source.length; index += 1) {
    const char = source[index];
    if (char === '\\') {
      let end = index + 1;
      while (end < source.length && /[a-zA-Z]/.test(source[end])) {
        end += 1;
      }
      const command = source.slice(index + 1, end);
      if (parenDepth === 0 && braceDepth === 0 && bracketDepth === 0) {
        if (command === 'le') {
          return { operator: '<=', left: source.slice(0, index), right: source.slice(end) };
        }
        if (command === 'ge') {
          return { operator: '>=', left: source.slice(0, index), right: source.slice(end) };
        }
        if (command === 'ne') {
          return { operator: '!=', left: source.slice(0, index), right: source.slice(end) };
        }
      }
      index = end - 1;
      continue;
    }

    if (char === '(') {
      parenDepth += 1;
      continue;
    }
    if (char === ')') {
      parenDepth -= 1;
      continue;
    }
    if (char === '{') {
      braceDepth += 1;
      continue;
    }
    if (char === '}') {
      braceDepth -= 1;
      continue;
    }
    if (char === '[') {
      bracketDepth += 1;
      continue;
    }
    if (char === ']') {
      bracketDepth -= 1;
      continue;
    }

    if (parenDepth !== 0 || braceDepth !== 0 || bracketDepth !== 0) {
      continue;
    }

    if (source.startsWith('<=', index)) {
      return { operator: '<=', left: source.slice(0, index), right: source.slice(index + 2) };
    }
    if (source.startsWith('>=', index)) {
      return { operator: '>=', left: source.slice(0, index), right: source.slice(index + 2) };
    }
    if (source.startsWith('!=', index)) {
      return { operator: '!=', left: source.slice(0, index), right: source.slice(index + 2) };
    }

    if (char === '<') {
      return { operator: '<', left: source.slice(0, index), right: source.slice(index + 1) };
    }
    if (char === '>') {
      return { operator: '>', left: source.slice(0, index), right: source.slice(index + 1) };
    }
    if (char === '=') {
      return { operator: '=', left: source.slice(0, index), right: source.slice(index + 1) };
    }
  }

  return null;
}

class LatexSubsetParser {
  private readonly tokens: Token[];
  private readonly options: ParserOptions;
  private readonly boundVariables: string[] = [];
  private index = 0;

  constructor(source: string, options: ParserOptions) {
    this.tokens = tokenize(source);
    this.options = options;
  }

  parse(): string {
    const expression = this.parseAdditive();
    this.expectEnd();
    return expression;
  }

  private parseAdditive(stopIndex?: number): string {
    let expression = this.parseMultiplicative(stopIndex);
    while (this.matchSymbol('+') || this.matchSymbol('-')) {
      const operator = this.previous().value;
      if (stopIndex !== undefined && this.index > stopIndex) {
        this.index -= 1;
        break;
      }
      expression = `(${expression}${operator}${this.parseMultiplicative(stopIndex)})`;
    }
    return expression;
  }

  private parseMultiplicative(stopIndex?: number): string {
    let expression = this.parsePower(stopIndex);
    while (true) {
      if (stopIndex !== undefined && this.index >= stopIndex) {
        break;
      }
      if (this.matchSymbol('*')) {
        expression = `(${expression}*${this.parsePower(stopIndex)})`;
        continue;
      }
      if (this.matchSymbol('/')) {
        expression = `(${expression}/${this.parsePower(stopIndex)})`;
        continue;
      }
      if (this.matchCommand('cdot') || this.matchCommand('times')) {
        expression = `(${expression}*${this.parsePower(stopIndex)})`;
        continue;
      }
      if (this.canStartImplicitFactor(stopIndex)) {
        expression = `(${expression}*${this.parsePower(stopIndex)})`;
        continue;
      }
      break;
    }
    return expression;
  }

  private parsePower(stopIndex?: number): string {
    let expression = this.parseUnary(stopIndex);
    if (this.matchSymbol('^')) {
      expression = `(${expression}^${this.parsePowerExponent()})`;
    }
    return expression;
  }

  private parseUnary(stopIndex?: number): string {
    if (this.matchSymbol('+')) {
      return this.parseUnary(stopIndex);
    }
    if (this.matchSymbol('-')) {
      return `(-${this.parseUnary(stopIndex)})`;
    }
    return this.parsePrimary(stopIndex);
  }

  private parsePrimary(stopIndex?: number): string {
    if (stopIndex !== undefined && this.index >= stopIndex) {
      throw new Error('Expected an expression before the bound-variable terminator.');
    }
    const token = this.peek();
    if (token.type === 'number') {
      this.advance();
      return token.value;
    }
    if (token.type === 'identifier') {
      return this.parseIdentifier(token.value);
    }
    if (token.type === 'command') {
      return this.parseCommand(token.value);
    }
    if (token.type === 'symbol' && (token.value === '(' || token.value === '{')) {
      this.advance();
      const close = token.value === '(' ? ')' : '}';
      const expression = this.parseAdditive(stopIndex);
      this.expectSymbol(close);
      return `(${expression})`;
    }
    if (token.type === 'symbol' && token.value === '|') {
      this.advance();
      const expression = this.parseAdditive(stopIndex);
      this.expectSymbol('|');
      return `abs(${expression})`;
    }
    throw new Error(`Unsupported or incomplete math near "${token.value || 'end of input'}".`);
  }

  private parseIdentifier(rawIdentifier: string): string {
    this.advance();
    const identifier = TEXT_FUNCTION_ALIASES.get(rawIdentifier) ?? rawIdentifier;

    if (BOUND_FUNCTIONS.has(identifier)) {
      return this.parseBoundFunction(identifier);
    }

    if (ENGINE_FUNCTIONS.has(identifier)) {
      return `${identifier}(${this.parseFunctionArgument(identifier)})`;
    }

    if (identifier === 'x' || identifier === 'pi' || identifier === 'e') {
      return identifier;
    }

    if (this.options.allowYVariable && identifier === 'y') {
      return identifier;
    }

    if (this.options.allowYVariable && /^[xy]{2,}$/.test(identifier)) {
      return identifier.split('').join('*');
    }

    if (!this.options.allowYVariable && /^x{2,}$/.test(identifier)) {
      return identifier.split('').join('*');
    }

    if (this.boundVariables.includes(identifier)) {
      return identifier;
    }

    throw new Error(`Unsupported symbol "${identifier}" in the current graphing input.`);
  }

  private parseCommand(command: string): string {
    if (UNSUPPORTED_COMMAND_MESSAGES.has(command)) {
      throw new Error(UNSUPPORTED_COMMAND_MESSAGES.get(command) ?? 'Unsupported notation.');
    }

    switch (command) {
      case 'pi':
        this.advance();
        return 'pi';
      case 'infty':
        this.advance();
        throw new Error(
          '\u221e (infinity) cannot be used as a numeric value here. '
          + 'Use a large finite number like 100 as an approximation, '
          + 'or enter the integral body using a finite upper bound.',
        );
      case 'int':
        return this.parseIntegralCommand();
      case 'sum':
      case 'prod':
        return this.parseAggregateCommand(command);
      case 'theta':
      case 'alpha':
      case 'beta':
      case 'gamma':
      case 'delta':
        throw new Error(`The symbol \\${command} is available visually, but it is not executable in the current graphing runtime.`);
      case 'sqrt':
        return this.parseSqrt();
      case 'frac':
        return this.parseFrac();
      case 'operatorname':
      case 'mathrm':
      case 'mathit':
        return this.parseNamedIdentifier(command);
      default: {
        const normalized = TEXT_FUNCTION_ALIASES.get(command) ?? command;
        if (BOUND_FUNCTIONS.has(normalized)) {
          this.advance();
          return this.parseBoundFunction(normalized);
        }
        if (ENGINE_FUNCTIONS.has(normalized)) {
          this.advance();
          return `${normalized}(${this.parseFunctionArgument(normalized)})`;
        }
        throw new Error(`"\\${command}" is not supported by the current graphing runtime.`);
      }
    }
  }

  private parseSqrt(): string {
    this.advance();
    let degree: string | null = null;
    if (this.matchSymbol('[')) {
      degree = this.parseAdditive();
      this.expectSymbol(']');
    }
    const argument = this.parseRequiredGroup();
    if (degree) {
      return `(${argument}^(1/(${degree})))`;
    }
    return `sqrt(${argument})`;
  }

  private parseFrac(): string {
    this.advance();
    const numerator = this.parseRequiredGroup();
    const denominator = this.parseRequiredGroup();
    return `((${numerator})/(${denominator}))`;
  }

  private parseNamedIdentifier(command: string): string {
    this.advance();
    this.expectSymbol('{');
    let text = '';
    while (!(this.peek().type === 'symbol' && this.peek().value === '}')) {
      const token = this.peek();
      if (token.type === 'end') {
        throw new Error(`Unclosed \\${command}{...} block.`);
      }
      if (token.type === 'identifier' || token.type === 'number') {
        text += token.value;
        this.advance();
        continue;
      }
      if (token.type === 'command' && token.value === 'pi') {
        text += 'pi';
        this.advance();
        continue;
      }
      throw new Error(`Unsupported content inside \\${command}{...}.`);
    }
    this.expectSymbol('}');
    return this.parseIdentifier(text);
  }

  private parseIntegralCommand(): string {
    this.advance();
    const { lower, upper } = this.parseScriptBounds('Integral');
    const differential = this.findIntegralDifferential();
    if (!differential) {
      throw new Error('Integral notation needs a differential such as dx or dt.');
    }

    this.boundVariables.push(differential.variable);
    const body = trimOuterParens(this.parseAdditive(differential.index));
    this.boundVariables.pop();

    if (this.index !== differential.index) {
      throw new Error('Integral notation needs a body before the differential.');
    }

    this.advance();
    return `int(${differential.variable},${lower},${upper},${body})`;
  }

  private parseAggregateCommand(name: 'sum' | 'prod' | string): string {
    this.advance();
    const { variable, lower, upper } = this.parseAggregateBounds(name);
    this.boundVariables.push(variable);
    const body = this.peek().type === 'symbol' && ['{', '('].includes(this.peek().value)
      ? this.parseRequiredGroup()
      : trimOuterParens(this.parseMultiplicative());
    this.boundVariables.pop();
    return `${name}(${variable},${lower},${upper},${body})`;
  }

  private parseScriptBounds(label: string): { lower: string; upper: string } {
    let lower: string | null = null;
    let upper: string | null = null;

    while (this.peek().type === 'symbol' && (this.peek().value === '_' || this.peek().value === '^')) {
      const operator = this.peek().value;
      this.advance();
      const expression = this.parseScriptExpression();
      if (operator === '_') {
        lower = expression;
      } else {
        upper = expression;
      }
    }

    if (!lower || !upper) {
      throw new Error(`${label} notation needs both lower and upper bounds.`);
    }

    return { lower, upper };
  }

  private parseAggregateBounds(name: string): { variable: string; lower: string; upper: string } {
    let lower: { variable: string; lower: string } | null = null;
    let upper: string | null = null;

    while (this.peek().type === 'symbol' && (this.peek().value === '_' || this.peek().value === '^')) {
      const operator = this.peek().value;
      this.advance();
      if (operator === '_') {
        lower = this.parseAggregateLowerScript();
      } else {
        upper = this.parseScriptExpression();
      }
    }

    if (!lower || !upper) {
      throw new Error(`${name} notation needs bounds like _{n=1}^{m}.`);
    }

    return { variable: lower.variable, lower: lower.lower, upper };
  }

  private parseAggregateLowerScript(): { variable: string; lower: string } {
    const close = this.matchSymbol('{') ? '}' : this.matchSymbol('(') ? ')' : null;
    const variable = this.parseBindingIdentifier();
    this.expectSymbol('=');
    const lower = trimOuterParens(close ? this.parseAdditiveAtDelimiter(close) : this.parseAdditive());
    if (close) {
      this.expectSymbol(close);
    }
    return { variable, lower };
  }

  private parseScriptExpression(): string {
    if (this.matchSymbol('{')) {
      const expression = trimOuterParens(this.parseAdditiveAtDelimiter('}'));
      this.expectSymbol('}');
      return expression;
    }
    if (this.matchSymbol('(')) {
      const expression = trimOuterParens(this.parseAdditiveAtDelimiter(')'));
      this.expectSymbol(')');
      return expression;
    }
    return trimOuterParens(this.parsePrimary());
  }

  private parseAdditiveAtDelimiter(close: string): string {
    let expression = this.parseMultiplicative();
    while (!(this.peek().type === 'symbol' && this.peek().value === close)) {
      if (!(this.matchSymbol('+') || this.matchSymbol('-'))) {
        break;
      }
      const operator = this.previous().value;
      expression = `(${expression}${operator}${this.parseMultiplicative()})`;
    }
    return expression;
  }

  private findIntegralDifferential(): { index: number; variable: string } | null {
    let parenDepth = 0;
    let braceDepth = 0;
    let bracketDepth = 0;

    for (let offset = this.index; offset < this.tokens.length; offset += 1) {
      const token = this.tokens[offset];
      if (token.type === 'symbol') {
        if (token.value === '(') {
          parenDepth += 1;
        } else if (token.value === ')') {
          parenDepth -= 1;
        } else if (token.value === '{') {
          braceDepth += 1;
        } else if (token.value === '}') {
          braceDepth -= 1;
        } else if (token.value === '[') {
          bracketDepth += 1;
        } else if (token.value === ']') {
          bracketDepth -= 1;
        }
        continue;
      }

      if (parenDepth !== 0 || braceDepth !== 0 || bracketDepth !== 0) {
        continue;
      }

      if (token.type === 'identifier' && /^d[a-zA-Z]+$/.test(token.value)) {
        return { index: offset, variable: token.value.slice(1) };
      }
    }

    return null;
  }

  private parseBoundFunction(name: string): string {
    this.expectSymbol('(');
    const variable = this.parseBindingIdentifier();
    this.expectSymbol(',');
    const lower = trimOuterParens(this.parseAdditive());
    this.expectSymbol(',');
    const upper = trimOuterParens(this.parseAdditive());
    this.expectSymbol(',');
    this.boundVariables.push(variable);
    const body = trimOuterParens(this.parseAdditive());
    this.boundVariables.pop();
    this.expectSymbol(')');
    return `${name}(${variable},${lower},${upper},${body})`;
  }

  private parseBindingIdentifier(): string {
    const token = this.peek();
    if (token.type === 'identifier') {
      this.advance();
      return token.value;
    }
    if (token.type === 'command' && token.value === 'pi') {
      throw new Error('Bound variables cannot be pi.');
    }
    throw new Error('Bound operators require a variable name as their first argument.');
  }

  private parseFunctionArgument(name: string): string {
    if (!this.canStartPrimary(this.peek())) {
      throw new Error(`Function "${name}" is missing an argument.`);
    }
    return trimOuterParens(this.parsePrimary());
  }

  private parseRequiredGroup(): string {
    const token = this.peek();
    if (token.type === 'symbol' && (token.value === '{' || token.value === '(')) {
      this.advance();
      const close = token.value === '{' ? '}' : ')';
      const expression = this.parseAdditive();
      this.expectSymbol(close);
      return trimOuterParens(expression);
    }
    throw new Error('Expected a grouped expression.');
  }

  private parsePowerExponent(): string {
    if (this.matchSymbol('{')) {
      const expression = this.parseAdditive();
      this.expectSymbol('}');
      return `(${expression})`;
    }
    return `(${this.parseUnary()})`;
  }

  private canStartImplicitFactor(stopIndex?: number): boolean {
    if (stopIndex !== undefined && this.index >= stopIndex) {
      return false;
    }
    const token = this.peek();
    if (!this.canStartPrimary(token)) {
      return false;
    }
    return !(token.type === 'command' && UNSUPPORTED_COMMAND_MESSAGES.has(token.value));
  }

  private canStartPrimary(token: Token): boolean {
    if (token.type === 'number' || token.type === 'identifier' || token.type === 'command') {
      return true;
    }
    return token.type === 'symbol' && ['(', '{', '|'].includes(token.value);
  }

  private expectSymbol(symbol: string): void {
    if (!this.matchSymbol(symbol)) {
      throw new Error(`Expected "${symbol}".`);
    }
  }

  private expectEnd(): void {
    if (this.peek().type !== 'end') {
      throw new Error(`Unexpected trailing input near "${this.peek().value}".`);
    }
  }

  private matchCommand(command: string): boolean {
    if (this.peek().type === 'command' && this.peek().value === command) {
      this.advance();
      return true;
    }
    return false;
  }

  private matchSymbol(symbol: string): boolean {
    if (this.peek().type === 'symbol' && this.peek().value === symbol) {
      this.advance();
      return true;
    }
    return false;
  }

  private previous(): Token {
    return this.tokens[this.index - 1];
  }

  private peek(): Token {
    return this.tokens[this.index] ?? { type: 'end', value: '' };
  }

  private advance(): Token {
    const token = this.peek();
    if (this.index < this.tokens.length - 1) {
      this.index += 1;
    }
    return token;
  }
}

function tokenize(source: string): Token[] {
  const tokens: Token[] = [];
  for (let index = 0; index < source.length; ) {
    const char = source[index];

    // Whitespace is a word separator (left by sanitizeLatex for spacing commands
    // like \, and \!) — skip it silently so "t dt" stays two tokens.
    if (/\s/.test(char)) {
      index += 1;
      continue;
    }

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

    if (char === '\\') {
      let end = index + 1;
      while (end < source.length && /[a-zA-Z]/.test(source[end])) {
        end += 1;
      }
      if (end === index + 1) {
        throw new Error(`Unsupported LaTeX escape near "${source.slice(index, index + 2)}".`);
      }
      tokens.push({ type: 'command', value: source.slice(index + 1, end) });
      index = end;
      continue;
    }

    if ('+-*/^_=,(){}[]|<>'.includes(char)) {
      tokens.push({ type: 'symbol', value: char });
      index += 1;
      continue;
    }

    if (char === 'π') {
      tokens.push({ type: 'command', value: 'pi' });
      index += 1;
      continue;
    }

    if (char === '∞') {
      tokens.push({ type: 'command', value: 'infty' });
      index += 1;
      continue;
    }

    // Unicode calculus symbols emitted by some virtual keyboards / copy-paste.
    if (char === '∫') {
      tokens.push({ type: 'command', value: 'int' });
      index += 1;
      continue;
    }
    if (char === '∑') {
      tokens.push({ type: 'command', value: 'sum' });
      index += 1;
      continue;
    }
    if (char === '∏') {
      tokens.push({ type: 'command', value: 'prod' });
      index += 1;
      continue;
    }
    if (char === '≤' || char === '⩽') {
      tokens.push({ type: 'symbol', value: '<' });
      tokens.push({ type: 'symbol', value: '=' });
      index += 1;
      continue;
    }
    if (char === '≥' || char === '⩾') {
      tokens.push({ type: 'symbol', value: '>' });
      tokens.push({ type: 'symbol', value: '=' });
      index += 1;
      continue;
    }
    if (char === '≠') {
      tokens.push({ type: 'symbol', value: '!' });
      tokens.push({ type: 'symbol', value: '=' });
      index += 1;
      continue;
    }

    if (char === '·' || char === '×') {
      tokens.push({ type: 'symbol', value: '*' });
      index += 1;
      continue;
    }

    throw new Error(`Unsupported character "${char}" in visual math input.`);
  }

  tokens.push({ type: 'end', value: '' });
  return tokens;
}
