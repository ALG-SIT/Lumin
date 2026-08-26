import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface DiscoveredTeacher {
  name: string;
  host: string;
  port: number;
  session_uuid: string;
}

export interface StudentJoinProps {
  onJoined: () => void;
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
      await invoke("student_join", {
        host: teacher.host,
        port: teacher.port,
        sessionId: teacher.session_uuid,
        joinCode,
      });
      onJoined();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleManualJoin = async () => {
    if (!manualIp || !manualPort || !joinCode) return;
    setError(null);
    try {
      await invoke("student_join", {
        host: manualIp,
        port: parseInt(manualPort, 10),
        sessionId: "",
        joinCode,
      });
      onJoined();
    } catch (e) {
      setError(String(e));
    }
  };

  const canJoin = joinCode.length === 4;

  return (
    <div className="student-join">
      <div className="join-header">
        <h2 className="join-title">教室に参加</h2>
        <button className="join-reset" type="button" onClick={onReset}>
          役割を選び直す
        </button>
      </div>

      <p className="join-subtitle">
        同じWi-Fiにいる先生を探しています…
      </p>

      <div className="join-card">
        {isBrowsing && teachers.length === 0 && (
          <p className="join-status">教室を検索中</p>
        )}

        {teachers.length > 0 && (
          <div className="teacher-section">
            <h3 className="section-label">見つかった教室</h3>
            <div className="teacher-list">
              {teachers.map((teacher) => (
                <div key={teacher.session_uuid} className="teacher-card">
                  <div className="teacher-info">
                    <span className="teacher-name">{teacher.name}</span>
                    <span className="teacher-meta">
                      {teacher.host}:{teacher.port}
                    </span>
                  </div>
                  <input
                    className="code-input"
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
                    className="join-button"
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

        <div className="manual-join">
          <h3 className="section-label">または、手動で入力</h3>
          <div className="manual-fields">
            <input
              className="manual-input"
              placeholder="IPアドレス"
              value={manualIp}
              onChange={(e) => setManualIp(e.target.value)}
            />
            <input
              className="manual-input"
              placeholder="ポート"
              type="number"
              value={manualPort}
              onChange={(e) => setManualPort(e.target.value)}
            />
            <input
              className="code-input"
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
          </div>
          <button
            className="join-button"
            type="button"
            onClick={handleManualJoin}
            disabled={!manualIp || !manualPort || !canJoin}
          >
            参加
          </button>
        </div>
      </div>

      {error && <p className="error-message">{error}</p>}

      <p className="privacy-note">
        教師へ共有されるのは、正誤・誤概念・ヒント回数・再回答結果だけです。
      </p>
    </div>
  );
}
