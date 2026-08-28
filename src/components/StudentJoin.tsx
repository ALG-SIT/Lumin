import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
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
  session_uuid: string;
}

export interface JoinResultPayload {
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

  useEffect(() => {
    setIsBrowsing(true);
    invoke<DiscoveredTeacher[]>("browse_teachers")
      .then(setTeachers)
      .catch((e) => {
        console.error("browse_teachers failed", e);
        setError(String(e));
      })
      .finally(() => setIsBrowsing(false));
  }, []);

  const filteredJoinCode = (value: string) =>
    value.replace(/\D/g, "").slice(0, 4);

  const handleJoin = async (teacher: DiscoveredTeacher) => {
    setError(null);
    try {
      const res = await invoke<JoinResultPayload>("student_join", {
        host: teacher.host,
        port: teacher.port,
        sessionId: teacher.session_uuid,
        joinCode,
      });
      onJoined(res);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleManualJoin = async () => {
    if (!manualIp || !manualPort || !joinCode) return;
    setError(null);
    try {
      const res = await invoke<JoinResultPayload>("student_join", {
        host: manualIp,
        port: parseInt(manualPort, 10),
        sessionId: "",
        joinCode,
      });
      onJoined(res);
    } catch (e) {
      setError(String(e));
    }
  };

  const canJoin = joinCode.length === 4;

  return (
    <div className={studentJoin}>
      <div className={joinHeader}>
        <h2 className={joinTitle}>教室に参加</h2>
        <button className={joinReset} type="button" onClick={onReset}>
          役割を選び直す
        </button>
      </div>

      <p className={joinSubtitle}>同じWi-Fiにいる先生を探しています…</p>

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
                      {teacher.host}:{teacher.port}
                    </span>
                  </div>
                  <input
                    className={`${codeInput} ${baseInputFocus}`}
                    type="text"
                    inputMode="numeric"
                    autoComplete="one-time-code"
                    placeholder="4桁の参加コード"
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
                    disabled={!canJoin}
                  >
                    参加
                  </button>
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
              value={manualIp}
              onChange={(e) => setManualIp(e.target.value)}
            />
            <input
              className={`${manualInput} ${baseInputFocus}`}
              placeholder="ポート"
              type="number"
              value={manualPort}
              onChange={(e) => setManualPort(e.target.value)}
            />
            <input
              className={`${codeInput} ${baseInputFocus}`}
              type="text"
              inputMode="numeric"
              autoComplete="one-time-code"
              placeholder="4桁の参加コード"
              value={joinCode}
              onChange={(e) => setJoinCode(filteredJoinCode(e.target.value))}
              maxLength={4}
            />
          </div>
          <button
            className={joinButton}
            type="button"
            onClick={handleManualJoin}
            disabled={!manualIp || !manualPort || !canJoin}
          >
            参加
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
