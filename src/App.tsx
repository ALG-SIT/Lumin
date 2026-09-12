import { useState } from "react";
import {
  app,
  appBar,
  brand,
  brandIcon,
  resetButton,
  roleBadge,
} from "./components/App.css.ts";
import { AppBarModelStatus } from "./components/AppBarModelStatus";
import { DemoFlow } from "./components/DemoFlow";
import { type LuminRole, RoleSelection } from "./components/RoleSelection";
import { Settings } from "./components/Settings";
import { type JoinResultPayload, StudentJoin } from "./components/StudentJoin";
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
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [studentJoined, setStudentJoined] = useState(false);
  const [studentSessionId, setStudentSessionId] = useState("");
  const [participantToken, setParticipantToken] = useState<string | null>(null);
  const [joinedQuiz, setJoinedQuiz] = useState<unknown>(null);

  const handleStudentJoined = (payload: JoinResultPayload) => {
    setParticipantToken(payload.participantToken);
    setJoinedQuiz(payload.quiz);
    setStudentSessionId(crypto.randomUUID());
    setStudentJoined(true);
  };

  const handleStudentReset = () => {
    setStudentJoined(false);
    setStudentSessionId("");
    setRole(null);
  };

  return (
    <div className={app}>
      <header className={appBar}>
        <span className={brand}>
          <SunIcon className={brandIcon} />
          Lumin
        </span>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "0.75rem",
            minWidth: 0,
          }}
        >
          {role && (
            <>
              <span className={roleBadge}>現在の役割: {ROLE_LABELS[role]}</span>
              <button
                type="button"
                className={resetButton}
                onClick={() => setRole(null)}
              >
                役割を切り替える
              </button>
            </>
          )}
          {/* The model applies to every role, so its status and the way in to
              change it sit outside the role-specific controls. */}
          <AppBarModelStatus onOpenSettings={() => setSettingsOpen(true)} />
        </div>
      </header>
      <main>
        {/* Settings covers the work area rather than replacing the role, so
            closing it returns to exactly where the teacher or student was. */}
        {settingsOpen && <Settings onClose={() => setSettingsOpen(false)} />}
        {!settingsOpen && !role && <RoleSelection onSelect={setRole} />}
        {!settingsOpen && role === "teacher" && <TeacherDashboard />}
        {!settingsOpen && role === "student" && !studentJoined && (
          <StudentJoin
            onJoined={handleStudentJoined}
            onReset={handleStudentReset}
          />
        )}
        {!settingsOpen && role === "student" && studentJoined && (
          <StudentQuiz
            sessionId={studentSessionId}
            participantToken={participantToken}
            quiz={
              (joinedQuiz as Parameters<typeof StudentQuiz>[0]["quiz"]) ??
              undefined
            }
            onComplete={() => setStudentJoined(false)}
          />
        )}
        {!settingsOpen && role === "demo" && (
          <DemoFlow onReset={() => setRole(null)} />
        )}
      </main>
    </div>
  );
}
