import { describe, it, expect } from "vitest";
import { formatDuration, formatTime, formatDate, formatPercentage, cleanExeName } from "./index";

describe("formatDuration", () => {
  it("formats seconds correctly", () => {
    expect(formatDuration(-10)).toBe("0s");
    expect(formatDuration(45)).toBe("45s");
    expect(formatDuration(60)).toBe("1m");
    expect(formatDuration(65)).toBe("1m 5s");
    expect(formatDuration(3600)).toBe("1h");
    expect(formatDuration(3665)).toBe("1h 1m");
    expect(formatDuration(7200)).toBe("2h");
  });
});

describe("formatTime", () => {
  it("formats timestamp into localized time", () => {
    const timeStr = formatTime(0);
    expect(timeStr).toMatch(/:/);
  });
});

describe("formatDate", () => {
  it("formats timestamp into localized date", () => {
    const dateStr = formatDate(0);
    expect(dateStr.length).toBeGreaterThan(0);
  });
});

describe("formatPercentage", () => {
  it("formats percentages correctly", () => {
    expect(formatPercentage(0, 0)).toBe("0%");
    expect(formatPercentage(50, 100)).toBe("50%");
    expect(formatPercentage(1, 3)).toBe("33%");
    expect(formatPercentage(2, 3)).toBe("67%");
  });
});

describe("cleanExeName", () => {
  it("removes .exe suffix", () => {
    expect(cleanExeName("chrome.exe")).toBe("chrome");
    expect(cleanExeName("Code.EXE")).toBe("Code");
    expect(cleanExeName("notepad")).toBe("notepad");
  });
});
