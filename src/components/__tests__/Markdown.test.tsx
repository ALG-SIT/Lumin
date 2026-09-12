import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Markdown } from "../Markdown";

afterEach(cleanup);

function html(text: string): string {
  const { container } = render(<Markdown text={text} />);
  return container.innerHTML;
}

describe("Markdown", () => {
  it("renders headings, lists, quotes and rules as elements", () => {
    render(
      <Markdown
        text={[
          "# 見出し",
          "",
          "段落です。",
          "",
          "- 一つ目",
          "- 二つ目",
          "",
          "1. 手順1",
          "2. 手順2",
          "",
          "> 引用文",
          "",
          "---",
        ].join("\n")}
      />,
    );

    expect(screen.getByRole("heading", { name: "見出し" }).tagName).toBe("H3");
    expect(screen.getByText("段落です。").tagName).toBe("P");
    expect(screen.getAllByRole("list")).toHaveLength(2);
    expect(screen.getAllByRole("listitem")).toHaveLength(4);
    expect(screen.getByText("引用文").closest("blockquote")).not.toBeNull();
    expect(screen.getByRole("separator")).toBeDefined();
  });

  it("resolves inline emphasis, code and strikethrough", () => {
    render(<Markdown text="**太字**と*斜体*と`コード`と~~取り消し~~。" />);
    expect(screen.getByText("太字").tagName).toBe("STRONG");
    expect(screen.getByText("斜体").tagName).toBe("EM");
    expect(screen.getByText("コード").tagName).toBe("CODE");
    expect(screen.getByText("取り消し").tagName).toBe("DEL");
  });

  it("keeps code spans and code blocks literal", () => {
    render(
      <Markdown text={"`**not bold**`\n\n```ts\nconst a = **1**;\n```"} />,
    );
    expect(screen.getByText("**not bold**").tagName).toBe("CODE");
    expect(screen.getByText("const a = **1**;")).toBeDefined();
    expect(document.querySelector("strong")).toBeNull();
  });

  it("shows an unterminated fence as code, which is what a streaming reply looks like", () => {
    render(<Markdown text={"説明:\n\n```\n途中まで"} />);
    expect(screen.getByText("途中まで").tagName).toBe("CODE");
  });

  it("renders pipe tables with their header row", () => {
    render(
      <Markdown
        text={["| 概念 | 件数 |", "| --- | --- |", "| 通分 | 6 |"].join("\n")}
      />,
    );
    expect(screen.getByRole("columnheader", { name: "概念" })).toBeDefined();
    expect(screen.getByRole("cell", { name: "通分" })).toBeDefined();
  });

  it("never turns model output into markup or unsafe links", () => {
    // Model output is data. HTML stays text, and only http(s)/mailto links
    // become anchors.
    const rendered = html('<img src=x onerror="alert(1)">**なお太字**');
    expect(rendered).not.toContain("<img");
    expect(screen.getByText(/<img src=x/)).toBeDefined();

    cleanup();
    render(
      <Markdown text="[安全](https://example.com) と [危険](javascript:alert(1))" />,
    );
    expect(
      screen.getByRole("link", { name: "安全" }).getAttribute("href"),
    ).toBe("https://example.com");
    expect(screen.queryByRole("link", { name: "危険" })).toBeNull();
  });

  it("keeps line breaks inside a paragraph", () => {
    const { container } = render(<Markdown text={"一行目\n二行目"} />);
    const paragraph = container.querySelector("p");
    expect(paragraph?.textContent).toBe("一行目\n二行目");
  });
});
