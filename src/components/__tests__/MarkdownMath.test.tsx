import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Markdown } from "../Markdown";
import { markdownMathBlock } from "../Markdown.css.ts";
import { GEMMA4_E4B_MATH_REPLY, GEMMA4_MATH_REPLY } from "./realReplies";

afterEach(cleanup);

/** KaTeX's own markup, which is how a rendered formula is recognised. */
function formulas(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll(".katex"));
}

/**
 * What the teacher actually sees.
 *
 * KaTeX also emits MathML carrying the original LaTeX, so a screen reader can
 * read the formula; that copy is hidden from sight and has to be dropped
 * before asking whether any raw notation is still on screen.
 */
function visibleText(el: HTMLElement): string {
  const copy = el.cloneNode(true) as HTMLElement;
  for (const hidden of copy.querySelectorAll(".katex-mathml")) hidden.remove();
  return copy.textContent ?? "";
}

describe("math in replies", () => {
  it("keeps the source available to a screen reader", () => {
    const { container } = render(<Markdown text="傾きは $\Delta y$ です。" />);
    const mathml = container.querySelector(".katex-mathml");
    expect(mathml).not.toBeNull();
    expect(mathml?.textContent).toContain("\\Delta");
  });

  it("renders an inline expression instead of showing its LaTeX", () => {
    const { container } = render(
      <Markdown text="傾きは $\Delta y / \Delta x$ で求めます。" />,
    );

    expect(formulas(container)).toHaveLength(1);
    // The delimiters and the backslashes are gone from the text.
    expect(visibleText(container)).not.toContain("$");
    expect(visibleText(container)).not.toContain("\\Delta");
    // The symbols themselves are present, typeset.
    expect(visibleText(container)).toContain("Δ");
    expect(container.textContent).toContain("傾きは");
    expect(container.textContent).toContain("で求めます。");
  });

  it("renders display math on its own line", () => {
    const { container } = render(
      <Markdown text={"変化の割合:\n\n$$a = \\frac{\\Delta y}{\\Delta x}$$"} />,
    );
    const rendered = formulas(container);
    expect(rendered).toHaveLength(1);
    expect(rendered[0].closest(`.${markdownMathBlock}`)).not.toBeNull();
    expect(container.querySelector(".katex-display")).not.toBeNull();
  });

  it("accepts the \\( \\) and \\[ \\] spellings too", () => {
    const { container } = render(
      <Markdown text={"式は \\(y = ax + b\\) です。\n\n\\[\na = 3\n\\]"} />,
    );
    expect(formulas(container)).toHaveLength(2);
    expect(visibleText(container)).not.toContain("\\(");
    expect(visibleText(container)).not.toContain("\\[");
  });

  it("leaves prose about money alone", () => {
    // A lone $ with spaces around it is not a delimiter; treating it as one
    // would swallow whole sentences into a formula.
    const { container } = render(
      <Markdown text="教材費は $5 から $10 に上がりました。" />,
    );
    expect(formulas(container)).toHaveLength(0);
    expect(container.textContent).toContain("$5 から $10");
  });

  it("keeps LaTeX inside code spans literal", () => {
    const { container } = render(
      <Markdown text={"`$\\Delta y$` と書きます。"} />,
    );
    expect(formulas(container)).toHaveLength(0);
    expect(screen.getByText("$\\Delta y$").tagName).toBe("CODE");
  });

  it("shows invalid math as an error rather than crashing on model output", () => {
    const { container } = render(<Markdown text="$\\frac{1}{$" />);
    // Rendered, not thrown: the source is generated text and may be malformed.
    expect(container.textContent).toBeDefined();
  });

  it("renders math inside a list item, where a plan's steps put it", () => {
    const { container } = render(
      <Markdown text={"- 傾き $a$ を求める\n- 切片 $b$ を求める"} />,
    );
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(formulas(container)).toHaveLength(2);
  });

  it("renders a real reply's formulas, including one fenced over three lines", () => {
    const { container } = render(<Markdown text={GEMMA4_MATH_REPLY} />);

    // Two inline in the opening sentence, one display block, one for $m$,
    // and two in each of the three list items.
    expect(formulas(container).length).toBeGreaterThanOrEqual(9);
    expect(container.querySelector(".katex-display")).not.toBeNull();
    expect(visibleText(container)).not.toContain("\\frac");
    expect(visibleText(container)).not.toContain("$$");

    // Math nested inside bold inside an ordered list still resolves.
    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(3);
    expect(items[1].querySelector("strong .katex")).not.toBeNull();
    expect(visibleText(items[1] as HTMLElement)).not.toContain("$");
  });

  it("renders the other model's single-line display block", () => {
    const { container } = render(<Markdown text={GEMMA4_E4B_MATH_REPLY} />);
    expect(container.querySelector(".katex-display")).not.toBeNull();
    expect(container.querySelector(`.${markdownMathBlock}`)).not.toBeNull();
    expect(visibleText(container)).not.toContain("$$");
    expect(visibleText(container)).toContain("傾き");
  });
});
