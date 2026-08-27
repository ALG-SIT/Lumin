import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ModelManager } from "./ModelManager";
import { TeacherSessionControl } from "./TeacherSessionControl";
import { LessonPlanEditor } from "./LessonPlanEditor";
import { TeacherChat } from "./TeacherChat";
import type { Quiz } from "./StudentQuiz";

export interface TeacherDashboardProps {
  onReset?: () => void;
}

type TeacherTab = "dashboard" | "session" | "lesson" | "chat";

const TABS: { id: TeacherTab; label: string }[] = [
  { id: "dashboard", label: "概要" },
  { id: "session", label: "セッション" },
  { id: "lesson", label: "レッスンプラン" },
  { id: "chat", label: "AIチャット" },
];

interface AnalysisEvent {
  id: string;
  participantToken: string;
  sessionId: string | null;
  questionID: string;
  concept: string;
  misconception: string | null;
  correct: boolean;
  hintCount: number;
  retrySuccess: boolean;
  submittedAt: number;
}

interface MisconceptionSummary {
  name: string;
  count: number;
  share: number;
}

interface ClassSummary {
  participantCount: number;
  responseCount: number;
  correctRate: number;
  retrySuccessRate: number;
  averageHints: number;
  misconceptions: MisconceptionSummary[];
}

function ChatView({
  classSummary,
  activeQuiz,
}: {
  classSummary: ClassSummary | null;
  activeQuiz: Quiz | null;
}) {
  return <TeacherChat classSummary={classSummary} activeQuiz={activeQuiz} />;
}

export function TeacherDashboard({ onReset }: TeacherDashboardProps) {
  const [tab, setTab] = useState<TeacherTab>("dashboard");
  const [sessionCode, setSessionCode] = useState<string | null>(null);
  const [summary, setSummary] = useState<ClassSummary | null>(null);
  const [events, setEvents] = useState<AnalysisEvent[]>([]);
  const [isLoadingDemo, setIsLoadingDemo] = useState(false);
  const [activeQuiz] = useState<Quiz | null>(null);

  const refreshSummary = async () => {
    try {
      const s = await invoke<ClassSummary>("get_class_summary");
      setSummary(s);
    } catch (e) {
      console.error("get_class_summary failed", e);
    }
  };

  useEffect(() => {
    let mounted = true;

    refreshSummary();

    const unlisten = listen<AnalysisEvent>("analysis-event", (event) => {
      if (!mounted) return;
      setEvents((prev) => [...prev, event.payload]);
      void refreshSummary();
    });

    return () => {
      mounted = false;
      void unlisten.then((u) => u());
    };
  }, []);

  const loadDemoData = async () => {
    setIsLoadingDemo(true);
    try {
      await invoke("load_demo_data");
      await refreshSummary();
    } catch (e) {
      console.error("load_demo_data failed", e);
    } finally {
      setIsLoadingDemo(false);
    }
  };

  const hasData = (summary?.responseCount ?? 0) > 0 || events.length > 0;

  const maxMisconceptionCount = Math.max(
    1,
    ...(summary?.misconceptions.slice(0, 5).map((m) => m.count) ?? [1])
  );

  return (
    <section className="teacher-layout" aria-label="先生ダッシュボード">
      <nav className="teacher-sidebar" aria-label="教師ナビゲーション">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            className={tab === t.id ? "active" : ""}
            onClick={() => setTab(t.id)}
            aria-current={tab === t.id ? "page" : undefined}
          >
            {t.label}
          </button>
        ))}
        {onReset && (
          <button type="button" onClick={onReset}>
            役割を切り替える
          </button>
        )}
      </nav>

      <div className="teacher-content">
        {/* 参加コード常時表示(セッション有効中・全タブ共通) */}
        {sessionCode && (
          <div className="join-code-banner" role="status" aria-live="polite">
            <span className="join-code-label">参加コード</span>
            <strong className="join-code-value">{sessionCode}</strong>
          </div>
        )}

        {tab === "dashboard" && (
          <div className="teacher-dashboard">
            <div className="teacher-dashboard-header">
              <div>
                <h2 className="teacher-dashboard-title">クラスの概要</h2>
                <p className="teacher-dashboard-subtitle">
                  正答率ではなく、なぜ迷ったかを見ます。
                </p>
              </div>
            </div>

            {!hasData ? (
              <div className="teacher-empty-state">
          <div className="teacher-empty-icon" aria-hidden>
            ☀
          </div>
          <h3 className="teacher-empty-title">回答を待っています</h3>
          <p className="teacher-empty-text">
            生徒が回答すると、解答本文を含まない分析結果だけがここに届きます。
          </p>
          <button
            type="button"
            className="primary-button"
            onClick={loadDemoData}
            disabled={isLoadingDemo}
          >
            {isLoadingDemo ? "読み込み中…" : "大会デモ用データを読み込む"}
          </button>
        </div>
      ) : (
        <>
          <div className="metric-grid">
            <div className="metric-card">
              <span className="metric-value">
                {summary?.participantCount ?? 0}
              </span>
              <span className="metric-label">参加者数</span>
              <span className="metric-note">匿名トークン</span>
            </div>
            <div className="metric-card">
              <span className="metric-value">
                {((summary?.correctRate ?? 0) * 100).toFixed(0)}%
              </span>
              <span className="metric-label">初回正解率</span>
              <span className="metric-note">
                全{summary?.responseCount ?? 0}回答
              </span>
            </div>
            <div className="metric-card">
              <span className="metric-value">
                {((summary?.retrySuccessRate ?? 0) * 100).toFixed(0)}%
              </span>
              <span className="metric-label">リトライ成功率</span>
              <span className="metric-note">ヒント利用後</span>
            </div>
            <div className="metric-card">
              <span className="metric-value">
                {(summary?.averageHints ?? 0).toFixed(1)}
              </span>
              <span className="metric-label">平均ヒント数</span>
              <span className="metric-note">1回答あたり</span>
            </div>
          </div>

          <div className="dashboard-card misconception-bars">
            <div className="dashboard-card-header">
              <div>
                <h3 className="dashboard-card-title">よくある誤概念 TOP 5</h3>
                <p className="dashboard-card-subtitle">
                  受信した最小化データから集計
                </p>
              </div>
            </div>
            {(summary?.misconceptions ?? []).length === 0 ? (
              <p className="dashboard-empty">まだ誤概念の集計がありません</p>
            ) : (
              (summary?.misconceptions ?? [])
                .slice(0, 5)
                .map((m, i) => (
                  <div key={m.name} className="bar-row">
                    <div className="bar-meta">
                      <span className="bar-label">{m.name}</span>
                      <span className="bar-count">
                        {m.count}件 · {(m.share * 100).toFixed(0)}%
                      </span>
                    </div>
                    <div className="bar-track">
                      <div
                        className="bar-fill"
                        style={{
                          width: `${(m.count / maxMisconceptionCount) * 100}%`,
                          backgroundColor:
                            i === 0 ? "var(--lumin-error)" : "var(--lumin-warning)",
                        }}
                      />
                    </div>
                  </div>
                ))
            )}
          </div>

          <div className="dashboard-card recent-signals">
            <h3 className="dashboard-card-title">最近のシグナル</h3>
            {events.length === 0 ? (
              <p className="dashboard-empty">まだ信号がありません</p>
            ) : (
              events
                .slice(-10)
                .reverse()
                .map((e) => (
                  <div key={e.id} className="signal-card">
                    <span
                      className={`signal-badge ${e.correct ? "correct" : "incorrect"}`}
                      aria-label={e.correct ? "正解" : "不正解"}
                    >
                      {e.correct ? "✓" : "✗"}
                    </span>
                    <div className="signal-body">
                      <span className="signal-concept">{e.concept}</span>
                      <span className="signal-misconception">
                        {e.misconception ?? "初回で理解"}
                      </span>
                    </div>
                    <span className="signal-token">{e.participantToken}</span>
                  </div>
                ))
            )}
          </div>

          <div className="demo-actions">
            <button
              type="button"
              className="secondary-button"
              onClick={loadDemoData}
              disabled={isLoadingDemo}
            >
              {isLoadingDemo ? "読み込み中…" : "大会デモ用データを読み込む"}
            </button>
            <p className="privacy-note">
              生徒の解答本文や氏名は表示・保存されません
            </p>
          </div>
        </>
      )}
          </div>
        )}
        {tab === "session" && (
          <TeacherSessionControl
            onSessionStarted={(code) => {
              setSessionCode(code);
              setTab("dashboard");
            }}
            onSessionEnded={() => setSessionCode(null)}
          />
        )}
        {tab === "lesson" && summary && (
          <LessonPlanEditor
            classSummary={{
              participantCount: summary.participantCount,
              responseCount: summary.responseCount,
              correctRate: summary.correctRate,
              retrySuccessRate: summary.retrySuccessRate,
              averageHints: summary.averageHints,
              misconceptions: summary.misconceptions.map((m) => ({
                concept: m.name,
                count: m.count,
              })),
            }}
            onAdopted={() => {}}
          />
        )}
        {tab === "lesson" && !summary && (
          <div className="card">
            <p style={{ color: "var(--lumin-text-secondary)" }}>
              クラスのデータが集まるまでレッスンプランは作成できません。
            </p>
          </div>
        )}
        {tab === "chat" && (
          <>
            <section className="model-manager-panel">
              <h2>AIモデル管理</h2>
              <ModelManager />
            </section>
            <ChatView classSummary={summary} activeQuiz={activeQuiz} />
          </>
        )}
      </div>
    </section>
  );
}
