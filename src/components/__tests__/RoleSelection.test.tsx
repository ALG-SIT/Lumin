import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { RoleSelection } from "../RoleSelection";

describe("RoleSelection", () => {
  it("renders app title and tagline", () => {
    render(<RoleSelection onSelect={() => {}} />);
    expect(screen.getByText("Lumin")).toBeDefined();
    expect(screen.getByText("理解を照らし、次の学びにつなげる")).toBeDefined();
  });

  it("renders three role buttons", () => {
    render(<RoleSelection onSelect={() => {}} />);
    expect(screen.getByText("先生として始める")).toBeDefined();
    expect(screen.getByText("生徒として参加")).toBeDefined();
    expect(screen.getByText("デモとして試す")).toBeDefined();
  });

  it("calls onSelect with correct role", () => {
    const onSelect = vi.fn();
    render(<RoleSelection onSelect={onSelect} />);
    fireEvent.click(screen.getByText("先生として始める"));
    expect(onSelect).toHaveBeenCalledWith("teacher");
    fireEvent.click(screen.getByText("生徒として参加"));
    expect(onSelect).toHaveBeenCalledWith("student");
    fireEvent.click(screen.getByText("デモとして試す"));
    expect(onSelect).toHaveBeenCalledWith("demo");
  });
});
