import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Markdown } from "../Markdown";
import {
  GEMMA3_PLAN_REPLY,
  GEMMA4_E4B_FOLLOW_UP_REPLY,
  GEMMA4_FOLLOW_UP_REPLY,
  GEMMA4_SUMMARY_REPLY,
} from "./realReplies";

afterEach(cleanup);

/** The rendered tree with styling stripped, so the snapshot stays readable. */
function structure(container: HTMLElement): string {
  return container.innerHTML
    .replace(/ class="[^"]*"/g, "")
    .replace(/></g, ">\n<");
}

describe("rendering real model replies", () => {
  it("renders a summary answer as a list, not as literal asterisks", () => {
    const { container } = render(<Markdown text={GEMMA4_SUMMARY_REPLY} />);

    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(3);
    expect(within(items[0]).getByText("誤概念の件数").tagName).toBe("STRONG");
    expect(items[0].textContent).toBe("誤概念の件数: 4件");
    // None of the Markdown punctuation survives as text.
    expect(container.textContent).not.toContain("*");
  });

  it("nests a sub-list under its parent item, as the follow-up reply uses", () => {
    render(<Markdown text={GEMMA4_FOLLOW_UP_REPLY} />);

    const lists = screen.getAllByRole("list");
    expect(lists).toHaveLength(2);
    // The two detail bullets belong inside the single top-level item.
    const topItems = within(lists[0]).getAllByRole("listitem", {});
    expect(lists[0].children).toHaveLength(1);
    expect(within(topItems[0]).getByRole("list")).toBe(lists[1]);
    expect(lists[1].children).toHaveLength(2);
    expect(
      screen.getByText(/具体的な練習問題を通じて理解度を確認する/),
    ).toBeDefined();
  });

  it("renders headings, bold labels and an ordered list from a longer reply", () => {
    render(<Markdown text={GEMMA3_PLAN_REPLY} />);

    expect(
      screen.getByRole("heading", {
        name: "分数の問題解決の練習 - 授業での取り組み",
      }).tagName,
    ).toBe("H4");
    const lists = screen.getAllByRole("list");
    expect(lists.map((l) => l.tagName)).toEqual(["UL", "OL"]);
    expect(within(lists[1]).getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("段階的な問題解決:").tagName).toBe("STRONG");
  });

  it("keeps two top-level items with their own sub-lists (E4B bullet width)", () => {
    render(<Markdown text={GEMMA4_E4B_FOLLOW_UP_REPLY} />);

    const lists = screen.getAllByRole("list");
    // One outer list plus one nested list per top-level item.
    expect(lists).toHaveLength(3);
    expect(lists[0].children).toHaveLength(2);
    expect(lists[1].children).toHaveLength(2);
    expect(lists[2].children).toHaveLength(1);
    expect(
      screen.getByText("「added numerators」の概念の再確認と演習:").tagName,
    ).toBe("STRONG");
    // The wider bullet marker must not leak into the item text.
    for (const item of screen.getAllByRole("listitem")) {
      expect(item.textContent?.startsWith("*")).toBe(false);
    }
  });

  it("produces this exact structure for the follow-up reply", () => {
    const { container } = render(<Markdown text={GEMMA4_FOLLOW_UP_REPLY} />);
    expect(structure(container)).toMatchInlineSnapshot(`
      "<div>
      <ul>
      <li>
      <strong>次回の授業でやるべきこと</strong>
      <ul>
      <li>「added numerators」に関する誤概念について、具体的な練習問題を通じて理解度を確認する。</li>
      <li>誤概念の理解を深めるための追加の練習や解説を行う。</li>
      </ul>
      </li>
      </ul>
      </div>"
    `);
  });
});
