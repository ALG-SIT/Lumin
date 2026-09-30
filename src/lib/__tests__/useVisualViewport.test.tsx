import { act, render } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useVisualViewport } from "../useVisualViewport";

function ViewportHarness() {
  useVisualViewport();
  return <input aria-label="answer" />;
}

describe("useVisualViewport", () => {
  afterEach(() => {
    Object.defineProperty(window, "visualViewport", {
      configurable: true,
      value: undefined,
    });
    vi.restoreAllMocks();
  });

  it("tracks keyboard viewport changes and removes listeners and CSS state", () => {
    let resize: (() => void) | undefined;
    let scroll: (() => void) | undefined;
    const viewport = {
      height: 600,
      offsetTop: 10,
      scale: 1,
      addEventListener: vi.fn((name: string, cb: () => void) => {
        if (name === "resize") resize = cb;
        if (name === "scroll") scroll = cb;
      }),
      removeEventListener: vi.fn(),
    };
    Object.defineProperty(window, "visualViewport", {
      configurable: true,
      value: viewport,
    });
    vi.stubGlobal(
      "requestAnimationFrame",
      vi.fn(() => 1),
    );
    vi.stubGlobal("cancelAnimationFrame", vi.fn());

    const { unmount } = render(<ViewportHarness />);
    const root = document.documentElement;
    expect(root.style.getPropertyValue("--viewport-height")).toBe("600px");
    expect(root.style.getPropertyValue("--viewport-top")).toBe("10px");

    viewport.height = 400;
    act(() => resize?.());
    expect(root).toHaveAttribute("data-keyboard-open");
    expect(requestAnimationFrame).toHaveBeenCalledOnce();

    act(() => scroll?.());
    expect(root.style.getPropertyValue("--viewport-height")).toBe("400px");
    unmount();
    expect(viewport.removeEventListener).toHaveBeenCalledTimes(2);
    expect(cancelAnimationFrame).toHaveBeenCalledWith(1);
    expect(root).not.toHaveAttribute("data-keyboard-open");
    expect(root.style.getPropertyValue("--viewport-height")).toBe("");
    expect(root.style.getPropertyValue("--viewport-top")).toBe("");
  });

  it("ignores pinch zoom and works when visualViewport is unavailable", () => {
    const viewport = {
      height: 400,
      offsetTop: 0,
      scale: 2,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    };
    Object.defineProperty(window, "visualViewport", {
      configurable: true,
      value: viewport,
    });
    const { unmount } = render(<ViewportHarness />);
    expect(
      document.documentElement.style.getPropertyValue("--viewport-height"),
    ).toBe("");
    unmount();

    Object.defineProperty(window, "visualViewport", {
      configurable: true,
      value: undefined,
    });
    const withoutViewport = render(<ViewportHarness />);
    expect(() => withoutViewport.unmount()).not.toThrow();
  });
});
