import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface QuizQuestion {
  id: string;
  prompt: string;
  concept: string;
}

export interface Quiz {
  id: string;
  title: string;
  subject: string;
  questions: QuizQuestion[];
}

export interface Student {
  id: string;
  joined_at: string;
}

export interface TeacherSessionControlProps {
  /** セッション開始成功時に参加コードを上位へ通知 */
  onSessionStarted: (joinCode: string) => void;
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
      onSessionStarted(code);
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
    <section className="session-control" aria-label="小テスト配信">
      <h2 className="session-title">小テスト配信</h2>

      {!sessionActive ? (
        <>
          <div className="session-card">
            <div className="session-card-header">
              <div className="session-icon-badge" aria-hidden="true">
                <svg viewBox="0 0 24 24" fill="currentColor">
                  <path d="M6 22q-.825 0-1.413-.588T4 20V4q0-.825.588-1.413T6 2h12q.825 0 1.413.588T20 4v16q0 .825-.588 1.413T18 22H6Zm0-2h12V4H6v16Zm2-2h8v-2H8v2Zm0-4h8v-2H8v2Zm0-4h5V8H8v2Z" />
                </svg>
              </div>
              <div>
                <h3 className="session-card-title">教材を選択</h3>
                <p className="session-card-subtitle">
                  授業に合う小テストを選べます
                </p>
              </div>
            </div>

            <div className="subject-groups">
              {subjects.map((subject) => (
                <div key={subject} className="subject-group">
                  <h4 className="subject-label">{subject}</h4>
                  <div className="quiz-picker">
                    {quizzes
                      .filter((q) => q.subject === subject)
                      .map((q) => (
                        <button
                          key={q.id}
                          type="button"
                          className={`quiz-card ${
                            selectedQuiz?.id === q.id ? "selected" : ""
                          }`}
                          onClick={() => setSelectedQuiz(q)}
                          aria-pressed={selectedQuiz?.id === q.id}
                        >
                          <span className="quiz-card-title">{q.title}</span>
                          <span className="quiz-card-meta">
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
              className="primary-button session-start-button"
            >
              {isLoading ? "開始中…" : "この小テストを配信"}
            </button>
          </div>

          {selectedQuiz && (
            <div className="session-card">
              <div className="selected-quiz-header">
                <div>
                  <span className="selected-quiz-subject">
                    {selectedQuiz.subject}
                  </span>
                  <h3 className="selected-quiz-title">
                    {selectedQuiz.title}
                  </h3>
                  <p className="selected-quiz-meta">
                    {selectedQuiz.questions.length}問
                  </p>
                </div>
              </div>

              <ol className="question-list">
                {selectedQuiz.questions.map((question, index) => (
                  <li key={question.id} className="question-list-item">
                    <span className="question-number">{index + 1}</span>
                    <div className="question-body">
                      <p className="question-prompt">{question.prompt}</p>
                      <span className="question-concept">
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
          <div className="session-card join-code-card">
            <span className="join-code-label">参加コード</span>
            <span className="join-code-value" aria-live="polite">
              {joinCode}
            </span>
            <p className="join-code-hint">
              生徒は同じWi-Fiからこのコードを入力して参加します
            </p>
          </div>

          <div className="session-card">
            <div className="connected-students-header">
              <h3 className="session-card-title">
                接続中の生徒 ({students.length})
              </h3>
            </div>

            {students.length === 0 ? (
              <p className="empty-state">まだ生徒が接続していません</p>
            ) : (
              <ul className="student-list">
                {students.map((student) => (
                  <li key={student.id} className="student-row">
                    <span>{formatStudentLabel(student)}</span>
                    <button
                      type="button"
                      onClick={() => kickStudent(student.id)}
                      className="student-kick-button"
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
            className="secondary-button session-end-button"
          >
            {isLoading ? "終了中…" : "配信を終了"}
          </button>
        </>
      )}

      {error && <p className="error-message session-error">{error}</p>}

      <div className="session-card session-info-card">
        <div className="session-info-row">
          <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M12 3C7.46 3 3.34 4.78.29 7.67c-.18.18-.29.43-.29.71 0 .28.11.53.29.71l11 11c.39.39 1.02.39 1.41 0l11-11c.18-.18.29-.43.29-.71 0-.28-.11-.53-.29-.71C20.66 4.78 16.54 3 12 3zm0 2c3.49 0 6.62 1.26 9 3.33L12 17.33 3 8.33C5.38 6.26 8.51 5 12 5z" />
          </svg>
          <div>
            <h4 className="session-info-title">教室内だけで接続</h4>
            <p className="session-info-text">
              同じWi-Fiまたは近距離の端末を自動検出し、暗号化された通信で接続します。インターネット接続は使いません。
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
