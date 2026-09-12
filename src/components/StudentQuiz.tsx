import { invoke } from "@tauri-apps/api/core";
import { useCallback, useState } from "react";
import { baseInputFocus } from "../styles/global.css.ts";
import {
  errorMessage,
  primaryButton,
  privacyNote,
  progressBar,
  progressFill,
  questionConcept,
  questionPrompt,
  quizHeader,
  secondaryButton,
} from "../styles/shared.css.ts";
import {
  answerInput,
  completionCard,
  completionIcon,
  completionMessage,
  completionTitle,
  feedback as feedbackBase,
  feedbackCorrect,
  feedbackIncorrect,
  hintActions,
  hintCard,
  hintGuard,
  hintHeader,
  hintLabel,
  hintText,
  progressBarInQuiz,
  questionCard,
  quizProgress,
  quizTitle,
  studentQuiz,
} from "./StudentQuiz.css.ts";

export interface QuizQuestion {
  id: string;
  prompt: string;
  concept: string;
  acceptedAnswers: string[];
  hints: string[];
  /** Rust側 QuizQuestion の必須フィールド(deserialize契約) */
  genericMisconception: string;
  explanation: string;
}

export interface Quiz {
  id: string;
  title: string;
  subject: string;
  questions: QuizQuestion[];
}

export interface StudentQuizProps {
  sessionId: string;
  quiz?: Quiz;
  /** サーバ /students/join 応答の実トークン(無ければ従来プレフィックス) */
  participantToken?: string | null;
  onComplete: () => void;
}

interface AnswerAnalysis {
  isCorrect: boolean;
  misconception: string | null;
}

const SAMPLE_QUIZ: Quiz = {
  id: "demo-quiz",
  title: "一次関数チェック",
  subject: "数学",
  questions: [
    {
      id: "lf-01",
      prompt: "y = 3x + 4 の傾きは？",
      concept: "一次関数の傾き",
      acceptedAnswers: ["3", "+3"],
      hints: [
        "y = mx + b の形で、x の係数 m が傾きです",
        "切片は 4 です。傾きはもう一つの数です",
      ],
      genericMisconception: "傾きの符号の読み落とし",
      explanation: "y = mx + b の x の係数 m が傾き。ここでは m = 3。",
    },
    {
      id: "lf-02",
      prompt: "y = 2x + 5 の切片は？",
      concept: "一次関数の切片",
      acceptedAnswers: ["5", "+5"],
      hints: [
        "y = mx + b の b が切片です",
        "x = 0 のときの y の値を考えてみましょう",
      ],
      genericMisconception: "切片の計算漏れ",
      explanation: "y = mx + b の定数項 b が切片。ここでは b = 5。",
    },
    {
      id: "lf-03",
      prompt: "点 (1, 5) を通り、傾きが 2 の直線の式は？",
      concept: "一次関数の式",
      acceptedAnswers: ["y=2x+3", "y = 2x + 3"],
      hints: [
        "y = 2x + b の形で表せます",
        "x = 1, y = 5 を代入して b を求めましょう",
      ],
      genericMisconception: "一次関数の式",
      explanation: "b = 5 - 2×1 = 3 より y = 2x + 3。",
    },
    {
      id: "lf-04",
      prompt: "x が 1 増えると y が 4 増える一次関数の傾きは？",
      concept: "変化の割合",
      acceptedAnswers: ["4", "+4"],
      hints: [
        "傾きは x の増加量に対する y の増加量です",
        "増加量の比を式にすると y の増加 / x の増加 です",
      ],
      genericMisconception: "変化量の分子と分母の逆転",
      explanation: "傾き = yの増加量 ÷ xの増加量 = 4 ÷ 1 = 4。",
    },
    {
      id: "lf-05",
      prompt: "y = -2x + 6 で x = 2 のときの y は？",
      concept: "式への代入",
      acceptedAnswers: ["2"],
      hints: [
        "x に 2 を代入して計算します",
        "-2 × 2 を先に計算し、最後に 6 を足します",
      ],
      genericMisconception: "符号の計算ミス",
      explanation: "y = -2×2 + 6 = -4 + 6 = 2。",
    },
  ],
};

export function StudentQuiz({
  sessionId,
  quiz: quizProp,
  participantToken,
  onComplete,
}: StudentQuizProps) {
  const quiz = quizProp ?? SAMPLE_QUIZ;

  const [questionIndex, setQuestionIndex] = useState(0);
  const [answer, setAnswer] = useState("");
  const [lastAnswer, setLastAnswer] = useState("");
  const [hintCount, setHintCount] = useState(0);
  const [feedback, setFeedback] = useState<"idle" | "correct" | "incorrect">(
    "idle",
  );
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [isComplete, setIsComplete] = useState(false);
  const [shownHints, setShownHints] = useState<string[]>([]);
  const [generatedHint, setGeneratedHint] = useState<string | null>(null);
  const [initialWasCorrect, setInitialWasCorrect] = useState<boolean | null>(
    null,
  );
  const [firstMisconception, setFirstMisconception] = useState<string | null>(
    null,
  );
  const [error, setError] = useState<string | null>(null);

  const question = quiz.questions[questionIndex];
  const totalQuestions = quiz.questions.length;
  const progressLabel = `${questionIndex + 1} / ${totalQuestions} 問題目`;

  const generateHint = useCallback(
    async (level: number, submittedAnswer = lastAnswer) => {
      try {
        const hint = await invoke<string>("generate_hint", {
          questionJson: JSON.stringify(question),
          studentAnswer: submittedAnswer,
          previousHints: shownHints.slice(-3),
          hintLevel: level,
        });
        setGeneratedHint(hint);
        setShownHints((previous) => [...previous, hint]);
      } catch (e) {
        console.error("generate_hint failed", e);
        setGeneratedHint(null);
        const fallback = question.hints[level - 1] ?? question.hints[0];
        if (fallback) setShownHints((previous) => [...previous, fallback]);
      }
    },
    [question, lastAnswer, shownHints],
  );

  const handleSubmit = async () => {
    if (isAnalyzing || !answer.trim()) return;
    setIsAnalyzing(true);
    setLastAnswer(answer);
    setError(null);

    try {
      const result = await invoke<AnswerAnalysis>("analyze_answer", {
        questionJson: JSON.stringify(question),
        studentAnswer: answer,
        hintLevel: hintCount + 1,
      });

      if (initialWasCorrect === null) {
        setInitialWasCorrect(result.isCorrect);
        setFirstMisconception(result.misconception);
      }

      if (result.isCorrect) {
        setFeedback("correct");
      } else {
        setFeedback("incorrect");
        setAnswer("");
        const nextHintCount = hintCount === 0 ? 1 : hintCount;
        setHintCount(nextHintCount);
        setGeneratedHint(null);
        await generateHint(nextHintCount, answer);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setIsAnalyzing(false);
    }
  };

  const handleNextHint = async () => {
    if (isAnalyzing || hintCount >= question.hints.length) return;
    const nextHintCount = hintCount + 1;
    setHintCount(nextHintCount);
    setFeedback("idle");
    setGeneratedHint(null);
    setIsAnalyzing(true);
    try {
      await generateHint(nextHintCount);
    } finally {
      setIsAnalyzing(false);
    }
  };

  const handleRetry = () => {
    setAnswer("");
    setFeedback("idle");
  };

  const sendAnalysisEvent = async (retrySuccess: boolean) => {
    const event = {
      // AnalysisEvent(Rust)の必須フィールド: id / submittedAt(秒)
      id: crypto.randomUUID(),
      participantToken: participantToken ?? `student-${sessionId.slice(0, 8)}`,
      sessionId: null,
      questionID: question.id,
      concept: question.concept,
      misconception: firstMisconception,
      correct: initialWasCorrect ?? false,
      hintCount,
      retrySuccess,
      submittedAt: Math.floor(Date.now() / 1000),
    };

    try {
      await invoke("send_analysis_event", {
        eventJson: JSON.stringify(event),
      });
    } catch (e) {
      // The backend may not yet expose this command; log only.
      console.error("send_analysis_event failed", e);
    }
  };

  const handleAdvance = async () => {
    const retrySuccess = initialWasCorrect === false;
    await sendAnalysisEvent(retrySuccess);

    if (questionIndex === totalQuestions - 1) {
      setIsComplete(true);
    } else {
      setQuestionIndex((i) => i + 1);
      setShownHints([]);
      setLastAnswer("");
      setAnswer("");
      setHintCount(0);
      setFeedback("idle");
      setGeneratedHint(null);
      setInitialWasCorrect(null);
      setFirstMisconception(null);
      setError(null);
    }
  };

  if (isComplete) {
    return (
      <div className={studentQuiz}>
        <div className={completionCard}>
          <div className={completionIcon} aria-hidden>
            ☀
          </div>
          <h2 className={completionTitle}>おつかれさまでした</h2>
          <p className={completionMessage}>
            考え直した過程も、学びの大切な一部です。
            <br />
            先生には匿名の分析結果だけが共有されました。
          </p>
          <button className={primaryButton} type="button" onClick={onComplete}>
            参加画面へ戻る
          </button>
        </div>
      </div>
    );
  }

  const currentHint =
    generatedHint ?? question.hints[hintCount - 1] ?? "ヒントはありません。";

  const submitDisabled =
    isAnalyzing || (feedback !== "correct" && answer.trim() === "");

  return (
    <div className={studentQuiz}>
      <div className={quizHeader}>
        <span className={quizTitle}>{quiz.title}</span>
        <span className={quizProgress}>{progressLabel}</span>
      </div>

      <div className={`${progressBar} ${progressBarInQuiz}`} aria-hidden="true">
        <div
          className={progressFill}
          style={{
            width: `${((questionIndex + 1) / totalQuestions) * 100}%`,
          }}
        />
      </div>

      <div className={questionCard}>
        <span className={questionConcept}>{question.concept}</span>
        <p className={questionPrompt}>{question.prompt}</p>

        <input
          className={`${answerInput} ${baseInputFocus}`}
          value={answer}
          onChange={(e) => setAnswer(e.target.value)}
          placeholder="答えを入力"
          disabled={isAnalyzing || feedback === "correct"}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !submitDisabled) {
              void handleSubmit();
            }
          }}
        />

        {feedback === "correct" && (
          <p className={`${feedbackBase} ${feedbackCorrect}`}>
            その考え方で正解です
          </p>
        )}
        {feedback === "incorrect" && (
          <p className={`${feedbackBase} ${feedbackIncorrect}`}>
            まだ少し違うようです。ヒントを手がかりにもう一度。
          </p>
        )}

        <button
          className={primaryButton}
          type="button"
          onClick={feedback === "correct" ? handleAdvance : handleSubmit}
          disabled={submitDisabled}
        >
          {isAnalyzing
            ? "分析中…"
            : feedback === "correct"
              ? questionIndex === totalQuestions - 1
                ? "結果を送って終了"
                : "次の問題へ"
              : "答えを確かめる"}
        </button>
      </div>

      {hintCount > 0 && feedback !== "correct" && (
        <div className={hintCard}>
          <div className={hintHeader}>
            <span className={hintLabel}>ヒント {hintCount}</span>
            <span className={hintGuard}>答えはまだ見せません</span>
          </div>
          <p className={hintText}>{currentHint}</p>
          <div className={hintActions}>
            <button
              className={secondaryButton}
              type="button"
              onClick={handleNextHint}
              disabled={isAnalyzing || hintCount >= question.hints.length}
            >
              次のヒント
            </button>
            <button
              className={secondaryButton}
              type="button"
              onClick={handleRetry}
              disabled={isAnalyzing}
            >
              もう一度答える
            </button>
          </div>
        </div>
      )}

      {error && <p className={errorMessage}>{error}</p>}

      <p className={privacyNote}>
        ローカル分析中：解答は端末外へ送信されません
      </p>
    </div>
  );
}
