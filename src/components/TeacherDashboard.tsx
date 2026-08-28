import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  joinCodeLabel,
  joinCodeValue,
  primaryButton,
  privacyNote,
  secondaryButton,
} from "../styles/shared.css.ts";
import { LessonPlanEditor } from "./LessonPlanEditor";
import { ModelManager } from "./ModelManager";
import type { Quiz } from "./StudentQuiz";
import { TeacherChat } from "./TeacherChat";
import {
  barCount,
  barFill,
  barLabel,
  barMeta,
  barRow,
  barTrack,
  card,
  dashboardCard,
  dashboardCardHeader,
  dashboardCardSubtitle,
  dashboardCardTitle,
  dashboardEmpty,
  demoActions,
  joinCodeBanner,
  joinCodeValueBanner,
  metricCard,
  metricGrid,
  metricLabel,
  metricNote,
  metricValue,
  sidebarButton,
  sidebarButtonActive,
  signalBadge,
  signalBadgeCorrect,
  signalBadgeIncorrect,
  signalBody,
  signalCard,
  signalConcept,
  signalMisconception,
  signalToken,
  teacherContent,
  teacherDashboard,
  teacherDashboardHeader,
  teacherDashboardSubtitle,
  teacherDashboardTitle,
  teacherEmptyIcon,
  teacherEmptyState,
  teacherEmptyText,
  teacherEmptyTitle,
  teacherLayout,
  teacherSidebar,
} from "./TeacherDashboard.css.ts";
import { TeacherSessionControl } from "./TeacherSessionControl";

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

  const refreshSummary = useCallback(async () => {
    try {
      const s = await invoke<ClassSummary>("get_class_summary");
      setSummary(s);
    } catch (e) {
      console.error("get_class_summary failed", e);
    }
  }, []);

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
  }, [refreshSummary]);

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
    ...(summary?.misconceptions.slice(0, 5).map((m) => m.count) ?? [1]),
  );

  return (
    <section className={teacherLayout} aria-label="先生ダッシュボード">
      <nav className={teacherSidebar} aria-label="教師ナビゲーション">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            className={
              tab === t.id
                ? `${sidebarButton} ${sidebarButtonActive}`
                : sidebarButton
            }
            onClick={() => setTab(t.id)}
            aria-current={tab === t.id ? "page" : undefined}
          >
            {t.label}
          </button>
        ))}
        {onReset && (
          <button type="button" className={sidebarButton} onClick={onReset}>
            役割を切り替える
          </button>
        )}
      </nav>

      <div className={teacherContent}>
        {/* 参加コード常時表示(セッション有効中・全タブ共通) */}
        {sessionCode && (
          <div className={joinCodeBanner} role="status" aria-live="polite">
            <span className={joinCodeLabel}>参加コード</span>
            <strong className={`${joinCodeValue} ${joinCodeValueBanner}`}>
              {sessionCode}
            </strong>
          </div>
        )}

        {tab === "dashboard" && (
          <div className={teacherDashboard}>
            <div className={teacherDashboardHeader}>
              <div>
                <h2 className={teacherDashboardTitle}>クラスの概要</h2>
                <p className={teacherDashboardSubtitle}>
                  正答率ではなく、なぜ迷ったかを見ます。
                </p>
              </div>
            </div>

            {!hasData ? (
              <div className={teacherEmptyState}>
                <div className={teacherEmptyIcon} aria-hidden>
                  ☀
                </div>
                <h3 className={teacherEmptyTitle}>回答を待っています</h3>
                <p className={teacherEmptyText}>
                  生徒が回答すると、解答本文を含まない分析結果だけがここに届きます。
                </p>
                <button
                  type="button"
                  className={primaryButton}
                  onClick={loadDemoData}
                  disabled={isLoadingDemo}
                >
                  {isLoadingDemo ? "読み込み中…" : "大会デモ用データを読み込む"}
                </button>
              </div>
            ) : (
              <>
                <div className={metricGrid}>
                  <div className={metricCard}>
                    <span className={metricValue}>
                      {summary?.participantCount ?? 0}
                    </span>
                    <span className={metricLabel}>参加者数</span>
                    <span className={metricNote}>匿名トークン</span>
                  </div>
                  <div className={metricCard}>
                    <span className={metricValue}>
                      {((summary?.correctRate ?? 0) * 100).toFixed(0)}%
                    </span>
                    <span className={metricLabel}>初回正解率</span>
                    <span className={metricNote}>
                      全{summary?.responseCount ?? 0}回答
                    </span>
                  </div>
                  <div className={metricCard}>
                    <span className={metricValue}>
                      {((summary?.retrySuccessRate ?? 0) * 100).toFixed(0)}%
                    </span>
                    <span className={metricLabel}>リトライ成功率</span>
                    <span className={metricNote}>ヒント利用後</span>
                  </div>
                  <div className={metricCard}>
                    <span className={metricValue}>
                      {(summary?.averageHints ?? 0).toFixed(1)}
                    </span>
                    <span className={metricLabel}>平均ヒント数</span>
                    <span className={metricNote}>1回答あたり</span>
                  </div>
                </div>

                <div className={`${dashboardCard} misconception-bars`}>
                  <div className={dashboardCardHeader}>
                    <div>
                      <h3 className={dashboardCardTitle}>
                        よくある誤概念 TOP 5
                      </h3>
                      <p className={dashboardCardSubtitle}>
                        受信した最小化データから集計
                      </p>
                    </div>
                  </div>
                  {(summary?.misconceptions ?? []).length === 0 ? (
                    <p className={dashboardEmpty}>
                      まだ誤概念の集計がありません
                    </p>
                  ) : (
                    (summary?.misconceptions ?? []).slice(0, 5).map((m, i) => (
                      <div key={m.name} className={barRow}>
                        <div className={barMeta}>
                          <span className={barLabel}>{m.name}</span>
                          <span className={barCount}>
                            {m.count}件 · {(m.share * 100).toFixed(0)}%
                          </span>
                        </div>
                        <div className={barTrack}>
                          <div
                            className={barFill}
                            style={{
                              width: `${(m.count / maxMisconceptionCount) * 100}%`,
                              backgroundColor:
                                i === 0
                                  ? "var(--lumin-error)"
                                  : "var(--lumin-warning)",
                            }}
                          />
                        </div>
                      </div>
                    ))
                  )}
                </div>

                <div className={`${dashboardCard} recent-signals`}>
                  <h3 className={dashboardCardTitle}>最近のシグナル</h3>
                  {events.length === 0 ? (
                    <p className={dashboardEmpty}>まだ信号がありません</p>
                  ) : (
                    events
                      .slice(-10)
                      .reverse()
                      .map((e) => (
                        <div key={e.id} className={signalCard}>
                          <span
                            role="img"
                            className={`${signalBadge} ${e.correct ? signalBadgeCorrect : signalBadgeIncorrect}`}
                            aria-label={e.correct ? "正解" : "不正解"}
                          >
                            {e.correct ? "✓" : "✗"}
                          </span>
                          <div className={signalBody}>
                            <span className={signalConcept}>{e.concept}</span>
                            <span className={signalMisconception}>
                              {e.misconception ?? "初回で理解"}
                            </span>
                          </div>
                          <span className={signalToken}>
                            {e.participantToken}
                          </span>
                        </div>
                      ))
                  )}
                </div>

                <div className={demoActions}>
                  <button
                    type="button"
                    className={secondaryButton}
                    onClick={loadDemoData}
                    disabled={isLoadingDemo}
                  >
                    {isLoadingDemo
                      ? "読み込み中…"
                      : "大会デモ用データを読み込む"}
                  </button>
                  <p className={privacyNote}>
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
          <div className={card}>
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
