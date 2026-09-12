import katex from "katex";
import "katex/dist/katex.min.css";
import type { ReactNode } from "react";
import {
  markdown,
  markdownBlockquote,
  markdownCode,
  markdownCodeBlock,
  markdownHr,
  markdownMath,
  markdownMathBlock,
  markdownTable,
} from "./Markdown.css.ts";

/**
 * Render a subset of Markdown as React elements.
 *
 * The model is asked to answer in Markdown, so the reply has to arrive as
 * headings, lists and emphasis rather than as the literal `##` and `**` the
 * teacher would otherwise read.
 *
 * Everything becomes React nodes, never raw HTML: a reply is model output and
 * is not trusted as markup. HTML in the source is shown as text, and links are
 * limited to http(s) and mailto.
 *
 * Supported: ATX headings, fenced code, blockquotes, ordered/unordered lists,
 * horizontal rules, pipe tables, LaTeX math, and inline code, bold, italic,
 * strikethrough and links. Anything else stays literal text, which is the
 * right outcome for an assistant reply: nothing is silently dropped.
 */
export function Markdown({ text }: { text: string }) {
  return <div className={markdown}>{renderBlocks(text)}</div>;
}

/** One list item, with the list nested under it if it has one. */
interface ListItem {
  text: string;
  children: ListBlock | null;
}

interface ListBlock {
  kind: "list";
  ordered: boolean;
  items: ListItem[];
}

/**
 * Render one piece of LaTeX.
 *
 * This is the one place the renderer sets HTML directly, and it is safe for a
 * specific reason: the markup is produced by KaTeX from the source string
 * rather than taken from the reply, and KaTeX's `trust` option is left off, so
 * the commands that can emit a URL or arbitrary attributes (\href, \url,
 * \includegraphics, the \html* family) are refused. Invalid math is rendered
 * as a red error rather than thrown, because the source is model output.
 */
function Formula({ tex, display }: { tex: string; display: boolean }) {
  const html = katex.renderToString(tex, {
    displayMode: display,
    // The source is model output: malformed math is shown in red rather than
    // taking the whole reply down, and unusual-but-harmless notation is not
    // worth a warning the teacher would see.
    throwOnError: false,
    strict: "ignore",
    // Default output keeps the MathML alongside the visual typesetting, so a
    // screen reader reads the formula instead of skipping it.
  });
  return (
    <span
      className={display ? markdownMathBlock : markdownMath}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: KaTeXが生成した組版のみ。trustは無効で、URLや任意属性を出すコマンドは拒否される
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

type Block =
  | { kind: "heading"; level: number; text: string }
  | { kind: "paragraph"; lines: string[] }
  | { kind: "code"; language: string; lines: string[] }
  | { kind: "quote"; lines: string[] }
  | ListBlock
  | { kind: "table"; header: string[]; rows: string[][] }
  | { kind: "math"; tex: string }
  | { kind: "rule" };

const HEADING = /^(#{1,6})\s+(.*)$/;
const FENCE = /^\s*```\s*([\w+-]*)\s*$/;
const RULE = /^\s*(?:-{3,}|\*{3,}|_{3,})\s*$/;
const QUOTE = /^\s*>\s?(.*)$/;
const BULLET = /^\s*[-*+]\s+(.*)$/;
const ORDERED = /^\s*\d+[.)]\s+(.*)$/;
const TABLE_DIVIDER = /^\s*\|?[\s:|-]*-[\s:|-]*\|?\s*$/;
/** `$$…$$` or `\[…\]`, opened at the start of a line. */
const MATH_OPEN = /^\s*(\$\$|\\\[)/;
/** The same delimiters closing a display block. */
const MATH_CLOSE = /(\$\$|\\\])\s*$/;

function renderBlocks(text: string): ReactNode[] {
  return parseBlocks(text.split("\n")).map((block, index) => {
    const key = `${index}-${block.kind}`;
    switch (block.kind) {
      case "heading": {
        // The bubble already carries the reply's own heading level, so the
        // in-reply hierarchy starts one level down.
        const Tag = `h${Math.min(block.level + 2, 6)}` as "h3";
        return <Tag key={key}>{renderInline(block.text)}</Tag>;
      }
      case "code":
        return (
          <pre key={key} className={markdownCodeBlock}>
            <code data-language={block.language || undefined}>
              {block.lines.join("\n")}
            </code>
          </pre>
        );
      case "quote":
        return (
          <blockquote key={key} className={markdownBlockquote}>
            {renderBlocks(block.lines.join("\n"))}
          </blockquote>
        );
      case "list":
        return renderList(block, key);
      case "table":
        return (
          <table key={key} className={markdownTable}>
            <thead>
              <tr>
                {block.header.map((cell, i) => (
                  // biome-ignore lint/suspicious/noArrayIndexKey: 表のセルは位置で決まる
                  <th key={`${i}-${cell}`}>{renderInline(cell)}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {block.rows.map((row, r) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: 表の行は位置で決まる
                <tr key={`${r}-${row.join("|")}`}>
                  {row.map((cell, c) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: 表のセルは位置で決まる
                    <td key={`${c}-${cell}`}>{renderInline(cell)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        );
      case "math":
        return <Formula key={key} tex={block.tex} display />;
      case "rule":
        return <hr key={key} className={markdownHr} />;
      default:
        return <p key={key}>{renderInline(block.lines.join("\n"))}</p>;
    }
  });
}

function renderList(block: ListBlock, key: string): ReactNode {
  const items = block.items.map((item, i) => (
    // biome-ignore lint/suspicious/noArrayIndexKey: 返答ごとに全体を解析し直すため、位置がそのまま同一性を表す
    <li key={`${i}-${item.text}`}>
      {renderInline(item.text)}
      {item.children && renderList(item.children, `${i}-sub`)}
    </li>
  ));
  return block.ordered ? (
    <ol key={key}>{items}</ol>
  ) : (
    <ul key={key}>{items}</ul>
  );
}

/** A list line, with the indent that decides how deeply it is nested. */
interface ListLine {
  indent: number;
  ordered: boolean;
  text: string;
}

function matchListLine(line: string): ListLine | null {
  const ordered = ORDERED.exec(line);
  const bullet = ordered ? null : BULLET.exec(line);
  const match = ordered ?? bullet;
  if (!match) return null;
  return {
    indent: (/^\s*/.exec(line)?.[0] ?? "").length,
    ordered: ordered != null,
    text: match[1],
  };
}

/**
 * Group a run of list lines by indent.
 *
 * Replies really do come back with nested bullets, so a deeper line belongs
 * under the item above it rather than beside it.
 */
function buildList(lines: ListLine[]): ListBlock {
  const base = lines[0].indent;
  const items: ListItem[] = [];
  let i = 0;

  while (i < lines.length) {
    if (lines[i].indent > base && items.length > 0) {
      const nested: ListLine[] = [];
      while (i < lines.length && lines[i].indent > base) {
        nested.push(lines[i]);
        i += 1;
      }
      items[items.length - 1].children = buildList(nested);
      continue;
    }
    items.push({ text: lines[i].text, children: null });
    i += 1;
  }

  return { kind: "list", ordered: lines[0].ordered, items };
}

function parseBlocks(lines: string[]): Block[] {
  const blocks: Block[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    if (line.trim() === "") {
      i += 1;
      continue;
    }

    const fence = FENCE.exec(line);
    if (fence) {
      const body: string[] = [];
      i += 1;
      // An unterminated fence is normal while a reply is still streaming, so
      // the rest of the text is shown as code rather than held back.
      while (i < lines.length && !FENCE.test(lines[i])) {
        body.push(lines[i]);
        i += 1;
      }
      i += 1;
      blocks.push({ kind: "code", language: fence[1], lines: body });
      continue;
    }

    const math = matchDisplayMath(lines, i);
    if (math) {
      blocks.push({ kind: "math", tex: math.tex });
      i = math.next;
      continue;
    }

    if (RULE.test(line)) {
      blocks.push({ kind: "rule" });
      i += 1;
      continue;
    }

    const heading = HEADING.exec(line);
    if (heading) {
      blocks.push({
        kind: "heading",
        level: heading[1].length,
        text: heading[2],
      });
      i += 1;
      continue;
    }

    if (QUOTE.test(line)) {
      const body: string[] = [];
      while (i < lines.length && QUOTE.test(lines[i])) {
        body.push(QUOTE.exec(lines[i])?.[1] ?? "");
        i += 1;
      }
      blocks.push({ kind: "quote", lines: body });
      continue;
    }

    if (isTableStart(lines, i)) {
      const header = tableCells(lines[i]);
      const rows: string[][] = [];
      i += 2;
      while (i < lines.length && lines[i].includes("|")) {
        rows.push(tableCells(lines[i]));
        i += 1;
      }
      blocks.push({ kind: "table", header, rows });
      continue;
    }

    if (matchListLine(line)) {
      const collected: ListLine[] = [];
      while (i < lines.length) {
        const item = matchListLine(lines[i]);
        if (item) {
          collected.push(item);
          i += 1;
          continue;
        }
        // An indented non-list line continues the item above rather than
        // ending the list.
        if (collected.length > 0 && /^\s{2,}\S/.test(lines[i])) {
          collected[collected.length - 1].text += `\n${lines[i].trim()}`;
          i += 1;
          continue;
        }
        break;
      }
      blocks.push(buildList(collected));
      continue;
    }

    const paragraph: string[] = [];
    while (i < lines.length && !isBlockStart(lines, i)) {
      paragraph.push(lines[i]);
      i += 1;
    }
    blocks.push({ kind: "paragraph", lines: paragraph });
  }

  return blocks;
}

/**
 * Display math starting at `lines[i]`, on one line or fenced over several.
 *
 * An unterminated block still renders, the way an unterminated code fence
 * does: a reply is read while it streams, and holding the math back until the
 * closing delimiter arrives would make it appear as raw LaTeX first.
 */
function matchDisplayMath(
  lines: string[],
  i: number,
): { tex: string; next: number } | null {
  const open = MATH_OPEN.exec(lines[i]);
  if (!open) return null;
  const rest = lines[i].slice((open.index ?? 0) + open[0].length);

  // Closed on the same line: `$$x = 1$$`.
  const closesHere = MATH_CLOSE.exec(rest);
  if (closesHere) {
    return { tex: rest.slice(0, closesHere.index).trim(), next: i + 1 };
  }

  const body = rest.trim() ? [rest] : [];
  let line = i + 1;
  while (line < lines.length && !MATH_CLOSE.test(lines[line])) {
    body.push(lines[line]);
    line += 1;
  }
  if (line < lines.length) {
    const last = lines[line].replace(MATH_CLOSE, "");
    if (last.trim()) body.push(last);
  }
  return { tex: body.join("\n").trim(), next: line + 1 };
}

/** True where a line ends the paragraph being collected. */
function isBlockStart(lines: string[], i: number): boolean {
  const line = lines[i];
  return (
    line.trim() === "" ||
    FENCE.test(line) ||
    RULE.test(line) ||
    HEADING.test(line) ||
    MATH_OPEN.test(line) ||
    QUOTE.test(line) ||
    BULLET.test(line) ||
    ORDERED.test(line) ||
    isTableStart(lines, i)
  );
}

/** A header row is only a table when the next line is the `|---|` divider. */
function isTableStart(lines: string[], i: number): boolean {
  return (
    lines[i].includes("|") &&
    lines[i + 1] != null &&
    lines[i + 1].includes("-") &&
    TABLE_DIVIDER.test(lines[i + 1])
  );
}

function tableCells(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((cell) => cell.trim());
}

// Code first, so a span quoting Markdown or LaTeX stays literal. Math next, so
// an expression like $\Delta y / \Delta x$ is handed to KaTeX whole instead of
// having its backslashes and underscores eaten by the emphasis rules.
//
// `$…$` deliberately refuses a space next to either delimiter, which is what
// keeps prose about money ("$5 から $10 へ") from being read as math.
const INLINE =
  /(`+)([\s\S]+?)\1|\$(?!\s)((?:[^$\n\\]|\\.)+?)(?<!\s)\$|\\\(([\s\S]+?)\\\)|\*\*([\s\S]+?)\*\*|__([\s\S]+?)__|~~([\s\S]+?)~~|(?<![\w*])\*([^*\n]+?)\*(?![\w*])|(?<![\w_])_([^_\n]+?)_(?![\w_])|\[([^\]]*)\]\(([^()\s]+)\)/;

const SAFE_LINK = /^(https?:\/\/|mailto:)/i;

/** Turn one run of text into nodes, resolving inline markup as it goes. */
function renderInline(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  let rest = text;
  let key = 0;

  while (rest.length > 0) {
    const match = INLINE.exec(rest);
    if (!match || match.index == null) {
      nodes.push(rest);
      break;
    }
    if (match.index > 0) {
      nodes.push(rest.slice(0, match.index));
    }

    const [
      ,
      ,
      code,
      dollarMath,
      parenMath,
      boldStar,
      boldUnderscore,
      strike,
      italicStar,
      italicUnderscore,
      linkText,
      href,
    ] = match;
    const id = `i${key++}`;
    // Code spans win: their content is literal, never markup.
    if (code != null) {
      nodes.push(
        <code key={id} className={markdownCode}>
          {code}
        </code>,
      );
    } else if (dollarMath != null || parenMath != null) {
      nodes.push(
        <Formula key={id} tex={dollarMath ?? parenMath} display={false} />,
      );
    } else if (boldStar != null || boldUnderscore != null) {
      nodes.push(
        <strong key={id}>{renderInline(boldStar ?? boldUnderscore)}</strong>,
      );
    } else if (strike != null) {
      nodes.push(<del key={id}>{renderInline(strike)}</del>);
    } else if (italicStar != null || italicUnderscore != null) {
      nodes.push(
        <em key={id}>{renderInline(italicStar ?? italicUnderscore)}</em>,
      );
    } else if (href != null && SAFE_LINK.test(href)) {
      nodes.push(
        <a key={id} href={href} target="_blank" rel="noreferrer noopener">
          {renderInline(linkText)}
        </a>,
      );
    } else {
      // An unusable scheme stays literal text rather than becoming a link.
      nodes.push(match[0]);
    }

    rest = rest.slice(match.index + match[0].length);
  }

  return nodes;
}
