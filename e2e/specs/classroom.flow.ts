import { browser, expect } from "@wdio/globals";

describe("Tauri teacher and student classroom flow", () => {
  it("delivers a hinted retry to the teacher and ends the session", async () => {
    const teacher = browser.teacher;
    const student = browser.student;

    await expect(teacher.$("h1")).toHaveText("Lumin");
    await teacher.$("button[aria-label='先生として始める画面を開く']").click();
    await expect(teacher.$("[aria-label='先生ダッシュボード']")).toExist();
    await teacher.$("button=セッション").click();
    await expect(teacher.$("section[aria-label='小テスト配信']")).toExist();
    const demoQuiz = teacher.$(
      "//button[contains(normalize-space(.), '一次関数 ミニチェック')]",
    );
    await expect(demoQuiz).toExist();
    await demoQuiz.click();
    await teacher.$("button=この小テストを配信").click();
    const joinCode = teacher.$("[aria-live='polite']");
    await expect(joinCode).toHaveText(/[0-9]{4}/);
    const code = await joinCode.getText();

    await expect(student.$("h1")).toHaveText("Lumin");
    await student.$("button[aria-label='生徒として参加画面を開く']").click();
    await student.$("input[aria-label='IPアドレス']").setValue("127.0.0.1");
    await student.$("input[aria-label='ポート']").setValue("8765");
    await student.$("input[aria-label='4桁の参加コード']").setValue(code);
    await student.$("button=参加").click();
    await expect(student.$("input[placeholder='答えを入力']")).toExist();
    await student.$("input[placeholder='答えを入力']").setValue("999");
    await student.$("button=答えを確かめる").click();
    await expect(student.$("span=答えはまだ見せません")).toExist();
    await student.$("button=もう一度答える").click();
    await student.$("input[placeholder='答えを入力']").setValue("3");
    await student.$("button=答えを確かめる").click();
    await expect(student.$("button=次の問題へ")).toExist();
    await student.$("button=次の問題へ").click();

    await teacher.$("button=概要").click();
    await expect(teacher.$("span=100%")).toExist();
    await teacher.$("button=セッション").click();
    await teacher.$("button=配信を終了").click();
    await expect(teacher.$("h3=教材を選択")).toExist();
  });
});
