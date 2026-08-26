import { useState } from "react";
import { DemoFlow } from "./components/DemoFlow";
import { RoleSelection, type LuminRole } from "./components/RoleSelection";
import { StudentJoin } from "./components/StudentJoin";
import { StudentQuiz } from "./components/StudentQuiz";
import { TeacherDashboard } from "./components/TeacherDashboard";

type Role = LuminRole | null;

const ROLE_LABELS: Record<LuminRole, string> = {
  teacher: "先生",
  student: "生徒",
  demo: "デモ",
};

function SunIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M12 7a5 5 0 100 10 5 5 0 000-10zM2 13h2a1 1 0 100-2H2a1 1 0 100 2zm18 0h2a1 1 0 100-2h-2a1 1 0 100 2zM11 2v2a1 1 0 102 0V2a1 1 0 10-2 0zm0 18v2a1 1 0 102 0v-2a1 1 0 10-2 0zM5.99 4.58a1 1 0 10-1.41 1.41l1.41 1.41a1 1 0 101.41-1.41L5.99 4.58zm12.37 12.37a1 1 0 10-1.41 1.41l1.41 1.41a1 1 0 101.41-1.41l-1.41-1.41zm1.41-10.96a1 1 0 10-1.41-1.41l-1.41 1.41a1 1 0 101.41 1.41l1.41-1.41zM7.4 17.39a1 1 0 10-1.41-1.41l-1.41 1.41a1 1 0 101.41 1.41l1.41-1.41z" />
    </svg>
  );
}

export default function App() {
  const [role, setRole] = useState<Role>(null);
  const [studentJoined, setStudentJoined] = useState(false);
  const [studentSessionId, setStudentSessionId] = useState("");

  const handleStudentJoined = () => {
    setStudentSessionId(crypto.randomUUID());
    setStudentJoined(true);
  };

  const handleStudentReset = () => {
    setStudentJoined(false);
    setStudentSessionId("");
    setRole(null);
  };

  return (
    <div className="app">
      <header className="app-bar">
        <span className="brand">
          <SunIcon className="brand-icon" />
          Lumin
        </span>
        {role && (
          <div style={{ display: "flex", alignItems: "center", gap: "0.75rem" }}>
            <span className="role-badge">
              現在の役割: {ROLE_LABELS[role]}
            </span>
            <button
              type="button"
              className="reset-button"
              onClick={() => setRole(null)}
            >
              役割を切り替える
            </button>
          </div>
        )}
      </header>
      <main>
        {!role && <RoleSelection onSelect={setRole} />}
        {role === "teacher" && <TeacherDashboard onReset={() => setRole(null)} />}
        {role === "student" && !studentJoined && (
          <StudentJoin
            onJoined={handleStudentJoined}
            onReset={handleStudentReset}
          />
        )}
        {role === "student" && studentJoined && (
          <StudentQuiz
            sessionId={studentSessionId}
            onComplete={() => setStudentJoined(false)}
          />
        )}
        {role === "demo" && <DemoFlow onReset={() => setRole(null)} />}
      </main>
    </div>
  );
}
