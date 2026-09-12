import { Channel, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { isTauriEnvironment } from "../lib/tauri";
import { baseInputFocus } from "../styles/global.css.ts";
import { ProgressRing, useElapsed } from "./GenerationProgress";
import { Markdown } from "./Markdown";
import type { ModelLoadProgress } from "./modelTypes";
import type { Quiz } from "./StudentQuiz";
import {
  messageAssistant,
  messageUser,
  teacherChat,
  teacherChatBubble,
  teacherChatChip,
  teacherChatComposer,
  teacherChatError,
  teacherChatFooter,
  teacherChatFooterNote,
  teacherChatHeader,
  teacherChatHistory,
  teacherChatInput,
  teacherChatInputRow,
  teacherChatMessage,
  teacherChatMessageRole,
  teacherChatPanel,
  teacherChatPrivacy,
  teacherChatSend,
  teacherChatSuggestions,
  teacherChatWelcome,
} from "./TeacherChat.css.ts";

export interface MisconceptionSummary {
  name: string;
  count: number;
  share: number;
}

export interface ClassSummary {
  participantCount: number;
  responseCount: number;
  correctRate: number;
  retrySuccessRate: number;
  averageHints: number;
  misconceptions: MisconceptionSummary[];
}

export interface TeacherChatProps {
  classSummary: ClassSummary | null;
  activeQuiz: Quiz | null;
}

interface Message {
  role: "user" | "assistant";
  content: string;
}

/**
 * One update from a running generation.
 *
 * `prefill` is the prompt being read into the cache: it is the whole wait
 * before any text exists, and on a long classroom context it is most of the
 * time the teacher spends waiting.
 */
type ChatChunk =
  | { kind: "prefill"; done: number; total: number }
  | { kind: "text"; text: string; generated: number };

/** What the chat is waiting on, for the progress panel. */
interface ChatPhase {
  stage: "prefill" | "generating";
  /** Prompt tokens read, for the prefill stage. */
  done?: number;
  total?: number;
  /** Reply tokens written, for the generating stage. */
  generated?: number;
}

const EMPTY_SUMMARY: ClassSummary = {
  participantCount: 0,
  responseCount: 0,
  correctRate: 0,
  retrySuccessRate: 0,
  averageHints: 0,
  misconceptions: [],
};

const SUGGESTIONS = [
  "何が分かっていない？",
  "次の手は？",
  "この誤概念への対処法は？",
  "ヒントの出し方を教えて",
];

function SendIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M3.5 13.09 20.5 4.5 12 20.5l-1.64-5.64L3.5 13.09z" />
    </svg>
  );
}

function LockIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M12 2a5 5 0 0 0-5 5v3H5v11h14V10h-2V7a5 5 0 0 0-5-5zm3 8H9V7a3 3 0 0 1 6 0v3z" />
    </svg>
  );
}

/**
 * Open a channel the backend can stream reply chunks over.
 *
 * Returns null outside a Tauri host (a plain browser, or tests), where there
 * is no IPC to carry one and the caller falls back to the whole-reply command.
 */
function chunkChannel(
  onChunk: (chunk: ChatChunk) => void,
): Channel<ChatChunk> | null {
  if (!isTauriEnvironment() || typeof Channel !== "function") return null;
  try {
    const channel = new Channel<ChatChunk>();
    channel.onmessage = onChunk;
    return channel;
  } catch {
    return null;
  }
}

/**
 * Ask the assistant, streaming the reply when the host supports it.
 *
 * The context carries the earlier turns with their speakers; the backend
 * replays them as real chat turns so the model can tell them from the
 * question being asked now.
 */
async function askAssistant(
  message: string,
  contextJson: string,
  onChunk: (chunk: ChatChunk) => void,
): Promise<string> {
  const channel = chunkChannel(onChunk);
  if (channel) {
    return invoke<string>("chat_with_teacher_stream", {
      message,
      contextJson,
      onChunk: channel,
    });
  }
  return invoke<string>("chat_with_teacher", { message, contextJson });
}

/** What the teacher is waiting on, said plainly. */
function phaseLabel(phase: ChatPhase | null, modelLoading: boolean): string {
  // Why the question has not started yet. How far the load has got is in the
  // app bar; repeating the number here would just be the same bar twice.
  if (modelLoading) return "モデルの読み込みを待っています…";
  if (!phase) return "質問を送信しました…";
  if (phase.stage === "prefill") {
    return `質問と授業の文脈を読み込み中… (${phase.done ?? 0}/${phase.total ?? 0} トークン)`;
  }
  return `回答を生成中… (${phase.generated ?? 0} トークン)`;
}

/**
 * Prompt processing has a known end, so it gets a real percentage.
 *
 * Decoding does not - it stops when the model stops, not at the token budget -
 * and it is only shown for the moment between the first token and the first
 * visible character anyway, so it reports no fraction rather than a
 * misleading one.
 */
function phasePercent(
  phase: ChatPhase | null,
  modelLoading: boolean,
): number | null {
  if (modelLoading) return null;
  if (phase?.stage === "prefill" && phase.total) {
    return ((phase.done ?? 0) / phase.total) * 100;
  }
  return null;
}

export function TeacherChat({ classSummary, activeQuiz }: TeacherChatProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [streamed, setStreamed] = useState("");
  const [phase, setPhase] = useState<ChatPhase | null>(null);
  // Only whether a load is in flight: how far along it is belongs to the app
  // bar, which owns model state for every screen.
  const [modelLoading, setModelLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const historyRef = useRef<HTMLDivElement>(null);
  const followLatest = useRef(true);

  const generationElapsed = useElapsed(loading);

  // biome-ignore lint/correctness/useExhaustiveDependencies: メッセージ・生成中テキスト・待機表示の更新時に履歴内だけを追従する
  useEffect(() => {
    const history = historyRef.current;
    if (messages.length > 0 && history && followLatest.current) {
      history.scrollTop = history.scrollHeight;
    }
  }, [messages, streamed, loading]);

  // Loading a model into memory takes tens of seconds even when the files are
  // already on disk, so it starts as soon as the chat opens and reports its
  // progress instead of stalling the teacher's first question.
  useEffect(() => {
    if (!isTauriEnvironment()) return;
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    listen<ModelLoadProgress>("model-load-progress", (event) => {
      if (cancelled) return;
      setModelLoading(!event.payload.done);
    }).then((stop) => {
      if (cancelled) stop();
      else unlisten = stop;
    });

    invoke<boolean>("preload_active_model")
      .catch((e) => {
        if (!cancelled) setError(`モデルの読み込みに失敗しました: ${e}`);
      })
      .finally(() => {
        if (!cancelled) setModelLoading(false);
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const send = useCallback(
    async (text: string) => {
      if (!text.trim() || loading) return;

      followLatest.current = true;
      const history = messages;
      setMessages((m) => [...m, { role: "user", content: text }]);
      setInput("");
      setStreamed("");
      setPhase(null);
      setLoading(true);
      setError(null);

      try {
        const context = {
          classSummary: classSummary ?? EMPTY_SUMMARY,
          activeQuiz,
          history,
        };

        const response = await askAssistant(
          text,
          JSON.stringify(context),
          (chunk) => {
            if (chunk.kind === "prefill") {
              setPhase({
                stage: "prefill",
                done: chunk.done,
                total: chunk.total,
              });
              return;
            }
            setPhase({ stage: "generating", generated: chunk.generated });
            if (chunk.text) setStreamed((current) => current + chunk.text);
          },
        );

        setMessages((m) => [...m, { role: "assistant", content: response }]);
      } catch (e) {
        setError(`エラー: ${e}`);
      } finally {
        setStreamed("");
        setPhase(null);
        setLoading(false);
      }
    },
    [activeQuiz, classSummary, loading, messages],
  );

  const canSend = input.trim().length > 0 && !loading;

  return (
    <section className={teacherChat} aria-label="AIと対話">
      <header className={teacherChatHeader}>
        <div>
          <h2>AI 先生方へ質問</h2>
          <p>
            現在の小テストと匿名集計を文脈にして、オンデバイスAIが回答します。
          </p>
        </div>
        <span className={teacherChatPrivacy}>
          <LockIcon style={{ width: 16, height: 16, flexShrink: 0 }} />
          個別の解答本文や氏名はAIへ渡しません
        </span>
      </header>

      {/* Transcript, suggestions and composer are one surface: two cards with
          a gap between them spent the height the conversation needs. */}
      <div className={teacherChatPanel}>
        <div
          ref={historyRef}
          onScroll={(event) => {
            const el = event.currentTarget;
            followLatest.current =
              el.scrollHeight - el.scrollTop - el.clientHeight < 48;
          }}
          className={teacherChatHistory}
          role="log"
          aria-live="polite"
          aria-label="会話履歴"
        >
          {messages.length === 0 && (
            <div className={teacherChatWelcome}>
              <h3>何を相談しますか？</h3>
              <p>
                {classSummary && classSummary.responseCount > 0
                  ? "正答率、再挑戦、ヒント利用、つまずきの傾向をもとに相談できます。"
                  : "回答はまだありません。教材の内容をもとに相談できます。"}
              </p>
            </div>
          )}

          {messages.map((msg, index) => (
            <div
              /* biome-ignore lint/suspicious/noArrayIndexKey: メッセージは追記のみでIDを持たないため */
              key={index}
              className={`${teacherChatMessage} ${msg.role === "user" ? messageUser : messageAssistant}`}
            >
              <span className={teacherChatMessageRole}>
                {msg.role === "user" ? "先生" : "AI"}
              </span>
              {msg.role === "user" ? (
                <p className={teacherChatBubble}>{msg.content}</p>
              ) : (
                <div className={teacherChatBubble}>
                  <Markdown text={msg.content} />
                </div>
              )}
            </div>
          ))}

          {loading && (
            <div className={`${teacherChatMessage} ${messageAssistant}`}>
              <span className={teacherChatMessageRole}>AI</span>
              <div className={teacherChatBubble}>
                {streamed ? (
                  <Markdown text={streamed} />
                ) : (
                  // The wait is drawn in the bubble the answer will fill, so
                  // nothing moves when it is replaced by the reply.
                  <ProgressRing
                    ariaLabel="回答の生成状況"
                    label={phaseLabel(phase, modelLoading)}
                    percent={phasePercent(phase, modelLoading)}
                    elapsedMs={generationElapsed}
                  />
                )}
              </div>
            </div>
          )}
        </div>

        <div className={teacherChatFooter}>
          <div className={teacherChatSuggestions}>
            {SUGGESTIONS.map((suggestion, index) => (
              <button
                /* biome-ignore lint/suspicious/noArrayIndexKey: 静的な候補リストのため */
                key={index}
                type="button"
                className={teacherChatChip}
                onClick={() => send(suggestion)}
                disabled={loading}
              >
                {suggestion}
              </button>
            ))}
          </div>

          <div className={teacherChatComposer}>
            {error && <p className={teacherChatError}>{error}</p>}
            <div className={teacherChatInputRow}>
              <textarea
                className={`${teacherChatInput} ${baseInputFocus}`}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={(e) => {
                  if (
                    e.key === "Enter" &&
                    !e.shiftKey &&
                    !e.nativeEvent.isComposing &&
                    e.keyCode !== 229
                  ) {
                    e.preventDefault();
                    send(input);
                  }
                }}
                placeholder="学習状況について質問"
                disabled={loading}
                rows={1}
              />
              <button
                type="button"
                className={teacherChatSend}
                onClick={() => send(input)}
                disabled={!canSend}
                aria-label="質問を送信"
              >
                <SendIcon />
              </button>
            </div>
            {/* The privacy line is already stated in the header with the lock;
                only the part the teacher has to act on is repeated here. */}
            <p className={teacherChatFooterNote}>
              提案は先生が確認して利用してください。
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
