import {
  appTitle,
  roleAction,
  roleButton,
  roleButtons,
  roleDetail,
  roleIcon,
  roleSelection,
  roleTitle,
  tagline,
  titleIcon,
  trustLabels,
} from "./RoleSelection.css.ts";

export type LuminRole = "teacher" | "student" | "demo";

export interface RoleSelectionProps {
  onSelect: (role: LuminRole) => void;
}

function SunIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M12 7a5 5 0 100 10 5 5 0 000-10zM2 13h2a1 1 0 100-2H2a1 1 0 100 2zm18 0h2a1 1 0 100-2h-2a1 1 0 100 2zM11 2v2a1 1 0 102 0V2a1 1 0 10-2 0zm0 18v2a1 1 0 102 0v-2a1 1 0 10-2 0zM5.99 4.58a1 1 0 10-1.41 1.41l1.41 1.41a1 1 0 101.41-1.41L5.99 4.58zm12.37 12.37a1 1 0 10-1.41 1.41l1.41 1.41a1 1 0 101.41-1.41l-1.41-1.41zm1.41-10.96a1 1 0 10-1.41-1.41l-1.41 1.41a1 1 0 101.41 1.41l1.41-1.41zM7.4 17.39a1 1 0 10-1.41-1.41l-1.41 1.41a1 1 0 101.41 1.41l1.41-1.41z" />
    </svg>
  );
}

function TeacherIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M4 4h16v2H4V4zm0 4h10v2H4V8zm0 4h16v2H4v-2zm0 4h10v2H4v-2zm0 4h16v2H4v-2z" />
    </svg>
  );
}

function StudentIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M3 17.25V21h3.75l11.06-11.06-3.75-3.75L3 17.25zm17.71-10.21a.996.996 0 000-1.41l-2.34-2.34a.996.996 0 00-1.41 0l-1.83 1.83 3.75 3.75 1.83-1.83z" />
    </svg>
  );
}

function DemoIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M8 5v14l11-7L8 5z" />
    </svg>
  );
}

function ArrowRightIcon(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true" {...props}>
      <path d="M12 4l-1.41 1.41L16.17 11H4v2h12.17l-5.58 5.59L12 20l8-8-8-8z" />
    </svg>
  );
}

interface RoleCardProps {
  role: LuminRole;
  title: string;
  detail: string;
  icon: React.ReactNode;
  onSelect: (role: LuminRole) => void;
}

function RoleCard({ role, title, detail, icon, onSelect }: RoleCardProps) {
  return (
    <button
      type="button"
      className={roleButton}
      onClick={() => onSelect(role)}
      aria-label={`${title}画面を開く`}
    >
      <span className={roleIcon}>{icon}</span>
      <p className={roleTitle}>{title}</p>
      <p className={roleDetail}>{detail}</p>
      <span className={roleAction}>
        続ける
        <ArrowRightIcon />
      </span>
    </button>
  );
}

export function RoleSelection({ onSelect }: RoleSelectionProps) {
  return (
    <section className={roleSelection} aria-labelledby="lumin-title">
      <h1 id="lumin-title" className={appTitle}>
        <SunIcon className={titleIcon} />
        Lumin
      </h1>
      <p className={tagline}>理解を照らし、次の学びにつなげる</p>
      <div className={roleButtons} role="group" aria-label="役割選択">
        <RoleCard
          role="teacher"
          title="先生として始める"
          detail="小テストを配信し、クラスのつまずきから次の10分を組み立てます。"
          icon={<TeacherIcon />}
          onSelect={onSelect}
        />
        <RoleCard
          role="student"
          title="生徒として参加"
          detail="自分のペースで解き、正解を見る前に段階的なヒントを受け取ります。"
          icon={<StudentIcon />}
          onSelect={onSelect}
        />
        <RoleCard
          role="demo"
          title="デモとして試す"
          detail="サンプルの問題でLuminの動作を体験できます。"
          icon={<DemoIcon />}
          onSelect={onSelect}
        />
        <ul className={trustLabels} aria-label="プライバシーの特徴">
          <li>インターネット接続なしで利用できます</li>
          <li>解答本文は生徒の端末から出ません</li>
          <li>最終判断をするのは先生です</li>
        </ul>
      </div>
    </section>
  );
}
