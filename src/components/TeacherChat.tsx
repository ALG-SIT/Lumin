import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { baseInputFocus } from "../styles/global.css.ts";
import type { Quiz } from "./StudentQuiz";
import {
  messageAssistant,
  messageUser,
  teacherChat,
  teacherChatBubble,
  teacherChatChip,
  teacherChatComposer,
  teacherChatError,
  teacherChatFooterNote,
  teacherChatHeader,
  teacherChatHistory,
  teacherChatInput,
  teacherChatInputRow,
  teacherChatMessage,
  teacherChatMessageRole,
  teacherChatPrivacy,
  teacherChatSend,
  teacherChatSuggestions,
  teacherChatThinking,
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

export function TeacherChat({ classSummary, activeQuiz }: TeacherChatProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const historyRef = useRef<HTMLDivElement>(null);
  const followLatest = useRef(true);

  // biome-ignore lint/correctness/useExhaustiveDependencies: メッセージと待機表示の更新時に履歴内だけを追従する
  useEffect(() => {
    const history = historyRef.current;
    if (messages.length > 0 && history && followLatest.current) {
      history.scrollTop = history.scrollHeight;
    }
  }, [messages, loading]);

  const send = async (text: string) => {
    if (!text.trim() || loading) return;

    followLatest.current = true;
    const userMsg: Message = { role: "user", content: text };
    setMessages((m) => [...m, userMsg]);
    setInput("");
    setLoading(true);
    setError(null);

    try {
      const recentMessages = messages.slice(-12).map((m) => {
        const label = m.role === "user" ? "先生" : "AI";
        return `${label}: ${m.content}`;
      });

      const context = {
        classSummary: classSummary ?? EMPTY_SUMMARY,
        activeQuiz,
        recentMessages,
      };

      const response = await invoke<string>("chat_with_teacher", {
        message: text,
        contextJson: JSON.stringify(context),
      });

      setMessages((m) => [...m, { role: "assistant", content: response }]);
    } catch (e) {
      const errText = `エラー: ${e}`;
      setError(errText);
    } finally {
      setLoading(false);
    }
  };

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
            <p className={teacherChatBubble}>{msg.content}</p>
          </div>
        ))}

        {loading && (
          <div className={`${teacherChatMessage} ${messageAssistant}`}>
            <span className={teacherChatMessageRole}>AI</span>
            <div className={teacherChatBubble}>
              <span className={teacherChatThinking}>考え中…</span>
            </div>
          </div>
        )}
      </div>

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
        <p className={teacherChatFooterNote}>
          個別の解答本文や氏名はAIへ渡しません。提案は先生が確認して利用してください。
        </p>
      </div>
    </section>
  );
}
