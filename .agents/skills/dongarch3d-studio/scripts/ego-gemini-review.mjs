// ego-browser (Ego Lite) 내 연결된 Google Gemini (Gemini Argon) 페이지에
// 현재 브랜치/커밋 변경사항 리뷰를 요청하고 응답을 추출하는 헬퍼 스크립트.
//
// 실행 방법 (PowerShell):
//   $scriptUri = (Resolve-Path ".agents\skills\dongarch3d-studio\scripts\ego-gemini-review.mjs").Path -replace '\\','/'
//   $env:STRATA_REVIEW_PROMPT = "리뷰 요청 프롬프트..."
//   ego-browser nodejs -e "await import('file:///$scriptUri')"

import { execSync } from "node:child_process";

let gitSummary = "";
try {
  gitSummary = execSync("git show HEAD --stat -U1", {
    cwd: "D:/gigamesh",
    encoding: "utf8",
    maxBuffer: 10 * 1024 * 1024,
  });
  if (gitSummary.length > 9000) {
    gitSummary = gitSummary.slice(0, 9000) + "\n...[diff truncated]";
  }
} catch (_) {
  gitSummary = "";
}

const promptText =
  process.env.STRATA_REVIEW_PROMPT ||
  `다음은 STRATA (strata-arch3d) 최신 커밋(git show HEAD) 변경사항입니다. STRATA Rust Core Rules(1: #![forbid(unsafe_code)], 2: 프로덕션 unwrap/expect 금지, 3: 1:1 치수 및 모든 회전 각도 단면·평면도·정밀 탁본 정합성, 4: UI 겹침 방지) 기준으로 코드 리뷰해 주세요:\n\n${gitSummary}`;

const spaces = await listTaskSpaces();
const geminiSpaceEntry =
  spaces.find((s) =>
    (s.recentTabTitles || []).some((t) => /Gemini/i.test(t))
  ) || spaces[0];

if (!geminiSpaceEntry) {
  throw new Error("ego-browser 내에 열려 있는 TaskSpace가 없습니다.");
}

const task = await claimTaskSpace(geminiSpaceEntry.id);
const tabs = await task.tabs();
const geminiTab =
  tabs.find((t) => /gemini\.google\.com/i.test(t.url || "")) ||
  tabs.find((t) => t.active) ||
  tabs[0];

if (!geminiTab) {
  throw new Error("ego-browser TaskSpace 내에 탭이 없습니다.");
}

const page = geminiTab.label
  ? task.page(geminiTab.label)
  : await task.adopt(geminiTab.page);

if (!/gemini\.google\.com/i.test(await page.url())) {
  await page.goto("https://gemini.google.com/app");
  await page.waitForLoadState("domcontentloaded");
}

// 전송 전 기존 응답 블록 수 및 마지막 응답 텍스트 기록
const beforeState = await page.evaluate(() => {
  const blocks = Array.from(
    document.querySelectorAll("model-response, message-content")
  );
  const lastBlock = blocks.length > 0 ? blocks[blocks.length - 1] : null;
  return {
    count: blocks.length,
    text: lastBlock ? (lastBlock.innerText || "").trim() : "",
  };
});

// Quill 에디터(.ql-editor) 안전한 멀티라인 주입 및 전송
const injected = await page.evaluate((rawText) => {
  const editor =
    document.querySelector(".ql-editor") ||
    document.querySelector('[contenteditable="true"][role="textbox"]');
  if (!editor) {
    return { ok: false, reason: "editor_not_found" };
  }
  editor.focus();
  while (editor.firstChild) {
    editor.removeChild(editor.firstChild);
  }
  for (const line of String(rawText || "").split(/\r?\n/)) {
    const p = document.createElement("p");
    if (line) {
      p.textContent = line;
    } else {
      p.appendChild(document.createElement("br"));
    }
    editor.appendChild(p);
  }
  const sel = window.getSelection();
  const range = document.createRange();
  range.selectNodeContents(editor);
  range.collapse(false);
  sel.removeAllRanges();
  sel.addRange(range);
  document.execCommand("insertText", false, " ");
  editor.dispatchEvent(
    new InputEvent("input", {
      bubbles: true,
      inputType: "insertText",
      data: " ",
    })
  );
  return { ok: true };
}, promptText);

if (!injected?.ok) {
  throw new Error(`Gemini 프롬프트 입력창 탐색 실패: ${JSON.stringify(injected)}`);
}

await page.waitForTimeout(500);
const clickedSend = await page.evaluate(() => {
  const btn = document.querySelector(
    'button[aria-label*="메시지 보내기"], button.send-button, button[aria-label*="보내기"], button[aria-label*="Send"], button[aria-label*="전송"]'
  );
  if (btn && !btn.disabled) {
    btn.click();
    return true;
  }
  return false;
});
if (!clickedSend) {
  await page.keyboard.press("Enter");
}

// 새 응답 블록이 생성되고 완료될 때까지 대기 (최대 90초)
let lastResponse = "";
let stableTicks = 0;
for (let step = 0; step < 45; step++) {
  await page.waitForTimeout(2000);
  const state = await page.evaluate(() => {
    const blocks = Array.from(
      document.querySelectorAll("model-response, message-content")
    );
    const lastBlock = blocks.length > 0 ? blocks[blocks.length - 1] : null;
    const text = lastBlock ? (lastBlock.innerText || "").trim() : "";
    const stopBtn = document.querySelector(
      'button[aria-label*="중지"], button[aria-label*="Stop"]'
    );
    return {
      count: blocks.length,
      text,
      generating: Boolean(stopBtn),
    };
  });

  const isNewResponse =
    state.count > beforeState.count ||
    (state.text && state.text !== beforeState.text);
  if (!isNewResponse) {
    continue;
  }

  if (state.text && state.text === lastResponse && !state.generating) {
    stableTicks += 1;
    if (stableTicks >= 2) {
      break;
    }
  } else {
    stableTicks = 0;
  }
  if (state.text) {
    lastResponse = state.text;
  }
}

console.log("=== [Gemini Argon Review Response] ===");
console.log(lastResponse || "(응답 추출 대기 중 — 스냅샷 확인)");
if (!lastResponse) {
  console.log(await page.snapshot());
}
