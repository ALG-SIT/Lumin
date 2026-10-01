import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { baseInputFocus } from "../styles/global.css.ts";
import { errorMessage, privacyNote } from "../styles/shared.css.ts";
import {
  codeInput,
  joinButton,
  joinCard,
  joinHeader,
  joinReset,
  joinStatus,
  joinSubtitle,
  joinTitle,
  manualFields,
  manualInput,
  manualJoin,
  sectionLabel,
  studentJoin,
  teacherCard,
  teacherInfo,
  teacherList,
  teacherMeta,
  teacherName,
  teacherSection,
} from "./StudentJoin.css.ts";

export interface DiscoveredTeacher {
  name: string;
  host: string;
  port: number;
  session_uuid: string | null;
  compatible?: boolean;
}

interface PendingJoin {
  teacher: DiscoveredTeacher;
  pendingId: string;
  teacherFingerprint: string;
}

export interface JoinResultPayload {
  sessionId?: string;
  participantToken: string | null;
  quiz: unknown | null;
}

export interface StudentJoinProps {
  /** 参加成功時、サーバ応答(token/配信中クイズ)を上位へ通知 */
  onJoined: (result: JoinResultPayload) => void;
  onReset: () => void;
}

export function StudentJoin({ onJoined, onReset }: StudentJoinProps) {
  const [teachers, setTeachers] = useState<DiscoveredTeacher[]>([]);
  const [isBrowsing, setIsBrowsing] = useState(true);
  const [manualIp, setManualIp] = useState("");
  const [manualPort, setManualPort] = useState("");
  const [joinCode, setJoinCode] = useState("");
  const [error, setError] = useState<string | null>(null);

  const [isJoining, setIsJoining] = useState(false);
  const [pending, setPending] = useState<PendingJoin | null>(null);
  const joining = useRef(false);

  const searchTeachers = useCallback(() => {
    setError(null);
    setIsBrowsing(true);
    invoke<DiscoveredTeacher[]>("browse_teachers")
      .then(setTeachers)
      .catch((e) => {
        console.error("browse_teachers failed", e);
        setError(String(e));
      })
      .finally(() => setIsBrowsing(false));
  }, []);

  useEffect(searchTeachers, [searchTeachers]);

  const filteredJoinCode = (value: string) =>
    value.replace(/\D/g, "").slice(0, 4);

  const handleJoin = async (teacher: DiscoveredTeacher) => {
    if (joining.current) return;
    if (teacher.compatible === false) {
      setError("この教室は旧版です。教師端末のアプリを更新してください。");
      return;
    }
    joining.current = true;
    setIsJoining(true);
    setError(null);
    try {
      if (pending) await invoke("student_cancel_connection");
      const prepared = await invoke<{ pendingId: string; teacherFingerprint: string }>("student_prepare_connection", {
        host: teacher.host,
        port: teacher.port,
        sessionId: teacher.session_uuid,
      });
      setPending({ teacher, ...prepared });
    } catch (e) {
      setError(String(e));
    } finally {
      joining.current = false;
      setIsJoining(false);
    }
  };

  const confirmJoin = async () => {
    if (!pending || joinCode.length !== 4 || joining.current) return;
    joining.current = true;
    setIsJoining(true);
    setError(null);
    try {
      const res = await invoke<JoinResultPayload>("student_join", {
        pendingId: pending.pendingId,
        teacherFingerprint: pending.teacherFingerprint,
        joinCode,
      });
      setPending(null);
      onJoined(res);
    } catch (e) {
      setError(String(e));
      setPending(null);
      await invoke("student_cancel_connection").catch(() => undefined);
    } finally {
      joining.current = false;
      setIsJoining(false);
    }
  };

  const cancelJoin = async () => {
    await invoke("student_cancel_connection").catch(() => undefined);
    setPending(null);
  };

  const validPort =
    /^\d+$/.test(manualPort) &&
    Number(manualPort) >= 1 &&
    Number(manualPort) <= 65535;
  const handleManualJoin = () => {
    if (!manualIp.trim() || !validPort || joinCode.length !== 4) return;
    return handleJoin({
      name: "",
      host: manualIp.trim(),
      port: Number(manualPort),
      session_uuid: null,
    });
  };

  const canJoin = joinCode.length === 4;
  const handleReset = () => {
    if (pending) void invoke("student_cancel_connection").catch(() => undefined);
    setPending(null);
    onReset();
  };

  return (
    <div className={studentJoin}>
      <div className={joinHeader}>
        <h2 className={joinTitle}>教室に参加</h2>
        <button className={joinReset} type="button" onClick={handleReset}>
          役割を選び直す
        </button>
      </div>

      <p className={joinSubtitle} role="status">
        {isBrowsing
          ? "同じWi-Fiにいる先生を探しています…"
          : teachers.length > 0
            ? "参加する教室を選んでください"
            : "教室が見つかりませんでした。同じWi-Fiか確認し、再検索または手動入力で参加できます。"}
      </p>
      <button
        type="button"
        className={joinReset}
        onClick={searchTeachers}
        disabled={isBrowsing || isJoining}
      >
        {isBrowsing ? "検索中…" : "教室を再検索"}
      </button>

      <div className={joinCard}>
        {isBrowsing && teachers.length === 0 && (
          <p className={joinStatus}>教室を検索中</p>
        )}

        {teachers.length > 0 && (
          <div className={teacherSection}>
            <h3 className={sectionLabel}>見つかった教室</h3>
            <div className={teacherList}>
              {teachers.map((teacher) => (
                <div key={teacher.session_uuid} className={teacherCard}>
                  <div className={teacherInfo}>
                    <span className={teacherName}>{teacher.name}</span>
                    <span className={teacherMeta}>
                      {teacher.host}:{teacher.port}{teacher.compatible === false ? " · 更新が必要" : ""}
                    </span>
                  </div>
                  <input
                    className={`${codeInput} ${baseInputFocus}`}
                    type="text"
                    inputMode="numeric"
                    autoComplete="one-time-code"
                    placeholder="4桁の参加コード"
                    aria-label="4桁の参加コード"
                    value={joinCode}
                    onChange={(e) =>
                      setJoinCode(filteredJoinCode(e.target.value))
                    }
                    maxLength={4}
                  />
                  <button
                    className={joinButton}
                    type="button"
                    onClick={() => handleJoin(teacher)}
                    disabled={!canJoin || isJoining || teacher.compatible === false}
                  >
                    {isJoining ? "参加中…" : "参加"}
                  </button>
                  {pending?.teacher.session_uuid === teacher.session_uuid && (
                    <div role="group" aria-label="教師の確認">
                      <p>教師画面の確認文字列と一致することを確認してください。</p>
                      <strong>{pending.teacherFingerprint}</strong>
                      <button type="button" onClick={confirmJoin} disabled={!canJoin || isJoining}>
                        一致を確認して参加
                      </button>
                      <button type="button" onClick={cancelJoin}>キャンセル</button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        <div className={manualJoin}>
          <h3 className={sectionLabel}>または、手動で入力</h3>
          <div className={manualFields}>
            <input
              className={`${manualInput} ${baseInputFocus}`}
              placeholder="IPアドレス"
              aria-label="IPアドレス"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
              value={manualIp}
              onChange={(e) => setManualIp(e.target.value)}
            />
            <input
              className={`${manualInput} ${baseInputFocus}`}
              placeholder="ポート"
              type="text"
              inputMode="numeric"
              aria-label="ポート"
              value={manualPort}
              onChange={(e) => setManualPort(e.target.value)}
            />
            <input
              className={`${codeInput} ${baseInputFocus}`}
              type="text"
              inputMode="numeric"
              autoComplete="one-time-code"
              placeholder="4桁の参加コード"
              aria-label="4桁の参加コード"
              value={joinCode}
              onChange={(e) => setJoinCode(filteredJoinCode(e.target.value))}
              maxLength={4}
            />
          </div>
          <button
            className={joinButton}
            type="button"
            onClick={handleManualJoin}
            disabled={!manualIp.trim() || !validPort || !canJoin || isJoining}
          >
            {isJoining ? "参加中…" : "参加"}
          </button>
        </div>
      </div>

      {error && <p className={errorMessage}>{error}</p>}

      <p className={privacyNote}>
        教師へ共有されるのは、正誤・誤概念・ヒント回数・再回答結果だけです。
      </p>
    </div>
  );
}
