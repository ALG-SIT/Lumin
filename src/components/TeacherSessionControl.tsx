import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import {
  emptyState,
  errorMessage,
  joinCodeLabel,
  joinCodeValue,
  primaryButton,
  questionConcept,
  questionPrompt,
  secondaryButton,
} from "../styles/shared.css.ts";
import type { Quiz as StudentQuiz } from "./StudentQuiz";
import {
  connectedStudentsHeader,
  joinCodeCard,
  joinCodeHint,
  questionBody,
  questionConceptInBody,
  questionList,
  questionListItem,
  questionNumber,
  questionPromptInBody,
  quizCard,
  quizCardMeta,
  quizCardSelected,
  quizCardTitle,
  quizPicker,
  selectedQuizHeader,
  selectedQuizMeta,
  selectedQuizSubject,
  selectedQuizTitle,
  sessionCard,
  sessionCardHeader,
  sessionCardSubtitle,
  sessionCardTitle,
  sessionControl,
  sessionEndButton,
  sessionError,
  sessionIconBadge,
  sessionInfoCard,
  sessionInfoRow,
  sessionInfoText,
  sessionInfoTitle,
  sessionStartButton,
  sessionTitle,
  studentKickButton,
  studentList,
  studentRow,
  subjectGroup,
  subjectGroups,
  subjectLabel,
} from "./TeacherSessionControl.css.ts";

export interface QuizQuestion {
  id: string;
  prompt: string;
  concept: string;
}

export interface Quiz extends StudentQuiz {
  id: string;
  title: string;
  subject: string;
}

export interface Student {
  id: string;
  joined_at: string;
}

export interface TeacherSessionControlProps {
  /** セッション開始成功時に参加コードを上位へ通知 */
  onSessionStarted: (joinCode: string, quiz: Quiz) => void;
  /** セッション終了時に通知(常時表示バナー解除用) */
  onSessionEnded?: () => void;
}

export function TeacherSessionControl({
  onSessionStarted,
  onSessionEnded,
}: TeacherSessionControlProps) {
  const [quizzes, setQuizzes] = useState<Quiz[]>([]);
  const [selectedQuiz, setSelectedQuiz] = useState<Quiz | null>(null);
  const [joinCode, setJoinCode] = useState<string | null>(null);
  const [students, setStudents] = useState<Student[]>([]);
  const [sessionActive, setSessionActive] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<Quiz[]>("list_quizzes")
      .then(setQuizzes)
      .catch((e) => setError(`クイズの読み込みに失敗しました: ${e}`));
  }, []);

  useEffect(() => {
    if (!sessionActive) return;

    const loadStudents = () => {
      invoke<Student[]>("list_students")
        .then(setStudents)
        .catch((e) => console.error("list_students failed", e));
    };

    loadStudents();
    const interval = setInterval(loadStudents, 2000);
    return () => clearInterval(interval);
  }, [sessionActive]);

  const subjects = Array.from(new Set(quizzes.map((q) => q.subject)));

  const startSession = async () => {
    if (!selectedQuiz) return;
    setIsLoading(true);
    setError(null);
    try {
      const code = await invoke<string>("start_session", {
        quizId: selectedQuiz.id,
      });
      setJoinCode(code);
      setSessionActive(true);
      onSessionStarted(code, selectedQuiz);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const endSession = async () => {
    setIsLoading(true);
    try {
      await invoke("end_session");
      onSessionEnded?.();
      setSessionActive(false);
      setJoinCode(null);
      setStudents([]);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const kickStudent = async (studentId: string) => {
    try {
      await invoke("kick_student", { studentId });
      setStudents((s) => s.filter((st) => st.id !== studentId));
    } catch (e) {
      setError(String(e));
    }
  };

  const formatStudentLabel = (student: Student) => {
    const short = student.id.slice(0, 6).toUpperCase();
    return `生徒 ${short}`;
  };

  return (
    <section className={sessionControl} aria-label="小テスト配信">
      <h2 className={sessionTitle}>小テスト配信</h2>

      {!sessionActive ? (
        <>
          <div className={sessionCard}>
            <div className={sessionCardHeader}>
              <div className={sessionIconBadge} aria-hidden="true">
                <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                  <path d="M6 22q-.825 0-1.413-.588T4 20V4q0-.825.588-1.413T6 2h12q.825 0 1.413.588T20 4v16q0 .825-.588 1.413T18 22H6Zm0-2h12V4H6v16Zm2-2h8v-2H8v2Zm0-4h8v-2H8v2Zm0-4h5V8H8v2Z" />
                </svg>
              </div>
              <div>
                <h3 className={sessionCardTitle}>教材を選択</h3>
                <p className={sessionCardSubtitle}>
                  授業に合う小テストを選べます
                </p>
              </div>
            </div>

            <div className={subjectGroups}>
              {subjects.map((subject) => (
                <div key={subject} className={subjectGroup}>
                  <h4 className={subjectLabel}>{subject}</h4>
                  <div className={quizPicker}>
                    {quizzes
                      .filter((q) => q.subject === subject)
                      .map((q) => (
                        <button
                          key={q.id}
                          type="button"
                          className={
                            selectedQuiz?.id === q.id
                              ? `${quizCard} ${quizCardSelected}`
                              : quizCard
                          }
                          onClick={() => setSelectedQuiz(q)}
                          aria-pressed={selectedQuiz?.id === q.id}
                        >
                          <span className={quizCardTitle}>{q.title}</span>
                          <span className={quizCardMeta}>
                            {q.questions.length}問
                          </span>
                        </button>
                      ))}
                  </div>
                </div>
              ))}
            </div>

            <button
              type="button"
              onClick={startSession}
              disabled={!selectedQuiz || isLoading}
              className={`${primaryButton} ${sessionStartButton}`}
            >
              {isLoading ? "開始中…" : "この小テストを配信"}
            </button>
          </div>

          {selectedQuiz && (
            <div className={sessionCard}>
              <div className={selectedQuizHeader}>
                <div>
                  <span className={selectedQuizSubject}>
                    {selectedQuiz.subject}
                  </span>
                  <h3 className={selectedQuizTitle}>{selectedQuiz.title}</h3>
                  <p className={selectedQuizMeta}>
                    {selectedQuiz.questions.length}問
                  </p>
                </div>
              </div>

              <ol className={questionList}>
                {selectedQuiz.questions.map((question, index) => (
                  <li key={question.id} className={questionListItem}>
                    <span className={questionNumber}>{index + 1}</span>
                    <div className={questionBody}>
                      <p
                        className={`${questionPrompt} ${questionPromptInBody}`}
                      >
                        {question.prompt}
                      </p>
                      <span
                        className={`${questionConcept} ${questionConceptInBody}`}
                      >
                        {question.concept}
                      </span>
                    </div>
                  </li>
                ))}
              </ol>
            </div>
          )}
        </>
      ) : (
        <>
          <div className={`${sessionCard} ${joinCodeCard}`}>
            <span className={joinCodeLabel}>参加コード</span>
            <span className={joinCodeValue} aria-live="polite">
              {joinCode}
            </span>
            <p className={joinCodeHint}>
              生徒は同じWi-Fiからこのコードを入力して参加します
            </p>
          </div>

          <div className={sessionCard}>
            <div className={connectedStudentsHeader}>
              <h3 className={sessionCardTitle}>
                接続中の生徒 ({students.length})
              </h3>
            </div>

            {students.length === 0 ? (
              <p className={emptyState}>まだ生徒が接続していません</p>
            ) : (
              <ul className={studentList}>
                {students.map((student) => (
                  <li key={student.id} className={studentRow}>
                    <span>{formatStudentLabel(student)}</span>
                    <button
                      type="button"
                      onClick={() => kickStudent(student.id)}
                      className={studentKickButton}
                    >
                      退出させる
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <button
            type="button"
            onClick={endSession}
            disabled={isLoading}
            className={`${secondaryButton} ${sessionEndButton}`}
          >
            {isLoading ? "終了中…" : "配信を終了"}
          </button>
        </>
      )}

      {error && <p className={`${errorMessage} ${sessionError}`}>{error}</p>}

      <div className={`${sessionCard} ${sessionInfoCard}`}>
        <div className={sessionInfoRow}>
          <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M12 3C7.46 3 3.34 4.78.29 7.67c-.18.18-.29.43-.29.71 0 .28.11.53.29.71l11 11c.39.39 1.02.39 1.41 0l11-11c.18-.18.29-.43.29-.71 0-.28-.11-.53-.29-.71C20.66 4.78 16.54 3 12 3zm0 2c3.49 0 6.62 1.26 9 3.33L12 17.33 3 8.33C5.38 6.26 8.51 5 12 5z" />
          </svg>
          <div>
            <h4 className={sessionInfoTitle}>教室内だけで接続</h4>
            <p className={sessionInfoText}>
              同じWi-Fiまたは近距離の端末を自動検出し、暗号化された通信で接続します。インターネット接続は使いません。
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
