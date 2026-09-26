import { useEffect } from "react";

/** WKWebView keeps its layout viewport tall when the software keyboard opens. */
export function useVisualViewport() {
  useEffect(() => {
    const viewport = window.visualViewport;
    if (!viewport) return;
    const root = document.documentElement;
    const update = () => {
      // Keep pinch zoom native; only follow keyboard/rotation viewport changes.
      if (viewport.scale !== 1) return;
      root.style.setProperty("--viewport-height", `${viewport.height}px`);
      root.style.setProperty("--viewport-top", `${viewport.offsetTop}px`);
      root.toggleAttribute(
        "data-keyboard-open",
        viewport.height < window.innerHeight - 100,
      );
    };
    let frame = 0;
    const resize = () => {
      if (viewport.scale !== 1) return;
      update();
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const field = document.activeElement;
        if (
          root.hasAttribute("data-keyboard-open") &&
          field instanceof HTMLInputElement
        ) {
          // Centre the field inside its scroll area, leaving room for the next action.
          field.scrollIntoView({ block: "center", inline: "nearest" });
        }
      });
    };
    update();
    viewport.addEventListener("resize", resize);
    viewport.addEventListener("scroll", update);
    return () => {
      viewport.removeEventListener("resize", resize);
      cancelAnimationFrame(frame);
      root.removeAttribute("data-keyboard-open");
      viewport.removeEventListener("scroll", update);
      root.style.removeProperty("--viewport-height");
      root.style.removeProperty("--viewport-top");
    };
  }, []);
}
