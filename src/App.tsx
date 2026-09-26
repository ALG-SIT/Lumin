import { useState } from "react";
import { app, appBar, workArea } from "./components/App.css.ts";
import { AppActions } from "./components/AppActions";
import { DemoFlow } from "./components/DemoFlow";
import { type LuminRole, RoleSelection } from "./components/RoleSelection";
import { Settings } from "./components/Settings";
import { type JoinResultPayload, StudentJoin } from "./components/StudentJoin";
import { StudentQuiz } from "./components/StudentQuiz";
import { TeacherDashboard } from "./components/TeacherDashboard";
import { useVisualViewport } from "./lib/useVisualViewport";

type Role = LuminRole | null;

const ROLE_LABELS: Record<LuminRole, string> = {
  teacher: "先生",
  student: "生徒",
  demo: "デモ",
};

export default function App() {
  useVisualViewport();
  const [role, setRole] = useState<Role>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [studentJoined, setStudentJoined] = useState(false);
  const [studentSessionId, setStudentSessionId] = useState("");
  const [participantToken, setParticipantToken] = useState<string | null>(null);
  const [joinedQuiz, setJoinedQuiz] = useState<unknown>(null);

  const handleStudentJoined = (payload: JoinResultPayload) => {
    setParticipantToken(payload.participantToken);
    setJoinedQuiz(payload.quiz);
    setStudentSessionId(payload.sessionId ?? "");
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
        <AppActions
          roleLabel={role ? ROLE_LABELS[role] : undefined}
          onSettings={() => setSettingsOpen(true)}
          onReset={() => {
            handleStudentReset();
            setSettingsOpen(false);
          }}
        />
      </header>
      <main>
        {/* Settings covers the work area rather than replacing the role, so
            closing it returns to exactly where the teacher or student was. */}
        {settingsOpen && <Settings onClose={() => setSettingsOpen(false)} />}
        <div className={workArea} hidden={settingsOpen}>
          {!role && <RoleSelection onSelect={setRole} />}
          {role === "teacher" && <TeacherDashboard />}
          {role === "student" && !studentJoined && (
            <StudentJoin
              onJoined={handleStudentJoined}
              onReset={handleStudentReset}
            />
          )}
          {role === "student" && studentJoined && (
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
          {role === "demo" && <DemoFlow onReset={() => setRole(null)} />}
        </div>
      </main>
    </div>
  );
}
