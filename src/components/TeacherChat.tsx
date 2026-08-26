import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Quiz } from "./StudentQuiz";

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
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  const send = async (text: string) => {
    if (!text.trim() || loading) return;

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
      setMessages((m) => [...m, { role: "assistant", content: errText }]);
    } finally {
      setLoading(false);
    }
  };

  const canSend = input.trim().length > 0 && !loading;

  return (
    <section className="teacher-chat" aria-label="AIと対話">
      <header className="teacher-chat-header">
        <div>
          <h2>AI 先生方へ質問</h2>
          <p>現在の小テストと匿名集計を文脈にして、オンデバイスAIが回答します。</p>
        </div>
        <span className="teacher-chat-privacy">
          <LockIcon style={{ width: 16, height: 16, flexShrink: 0 }} />
          個別の解答本文や氏名はAIへ渡しません
        </span>
      </header>

      <div className="teacher-chat-history" role="log" aria-live="polite" aria-label="会話履歴">
        {messages.length === 0 && (
          <div className="teacher-chat-welcome">
            <h3>何を相談しますか？</h3>
            <p>
              {classSummary && classSummary.responseCount > 0
                ? "正答率、再挑戦、ヒント利用、つまずきの傾向をもとに相談できます。"
                : "回答はまだありません。教材の内容をもとに相談できます。"}
            </p>
          </div>
        )}

        {messages.map((msg, index) => (
          <div key={index} className={`teacher-chat-message ${msg.role}`}>
            <span className="teacher-chat-message-role">
              {msg.role === "user" ? "先生" : "AI"}
            </span>
            <p className="teacher-chat-bubble">{msg.content}</p>
          </div>
        ))}

        {loading && (
          <div className="teacher-chat-message assistant">
            <span className="teacher-chat-message-role">AI</span>
            <div className="teacher-chat-bubble">
              <span className="teacher-chat-thinking">考え中…</span>
            </div>
          </div>
        )}
        <div ref={bottomRef} />
      </div>

      <div className="teacher-chat-suggestions">
        {SUGGESTIONS.map((suggestion, index) => (
          <button
            key={index}
            type="button"
            className="teacher-chat-chip"
            onClick={() => send(suggestion)}
            disabled={loading}
          >
            {suggestion}
          </button>
        ))}
      </div>

      <div className="teacher-chat-composer">
        {error && <p className="teacher-chat-error">{error}</p>}
        <div className="teacher-chat-input-row">
          <textarea
            className="teacher-chat-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
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
            className="teacher-chat-send"
            onClick={() => send(input)}
            disabled={!canSend}
            aria-label="質問を送信"
          >
            <SendIcon />
          </button>
        </div>
        <p className="teacher-chat-footer-note">
          個別の解答本文や氏名はAIへ渡しません。提案は先生が確認して利用してください。
        </p>
      </div>
    </section>
  );
}
