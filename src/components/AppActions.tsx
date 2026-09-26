import { useLayoutEffect, useRef, useState } from "react";
import { secondaryButton } from "../styles/shared.css.ts";
import {
  actionDialog,
  actionTrigger,
  dialogHeading,
  iconButton,
  roleHome,
  toolbarTitle,
} from "./App.css.ts";
import { AppBarModelStatus } from "./AppBarModelStatus";
import { CloseIcon, MoreIcon, RolesIcon } from "./NavigationIcons";

export function AppActions({
  roleLabel,
  onSettings,
  onReset,
}: {
  roleLabel?: string;
  onSettings: () => void;
  onReset: () => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const [open, setOpen] = useState(false);
  const close = () => dialog.current?.close();
  useLayoutEffect(() => {
    // Open after the controls mount so focus goes to the close button.
    if (open && !dialog.current?.open) dialog.current?.showModal();
  }, [open]);
  return (
    <>
      {roleLabel ? (
        <button
          type="button"
          className={roleHome}
          onClick={onReset}
          aria-label="役割選択へ戻る"
          title="役割選択へ戻る"
        >
          <RolesIcon />
          <span>役割</span>
        </button>
      ) : (
        <span className={roleHome} aria-hidden="true" />
      )}
      <span className={toolbarTitle}>{roleLabel ?? "Lumin"}</span>
      <button
        type="button"
        className={actionTrigger}
        aria-haspopup="dialog"
        aria-label="操作"
        title="設定とモデル"
        onClick={() => {
          setOpen(true);
        }}
      >
        <MoreIcon />
      </button>
      <dialog
        ref={dialog}
        className={actionDialog}
        aria-labelledby="app-actions-title"
        onClose={() => setOpen(false)}
      >
        {open && (
          <>
            <div className={dialogHeading}>
              <h2 id="app-actions-title">Luminの操作</h2>
              <button
                type="button"
                className={iconButton}
                onClick={close}
                aria-label="閉じる"
                title="閉じる"
              >
                <CloseIcon />
              </button>
            </div>
            {roleLabel && <p>現在の役割: {roleLabel}</p>}
            <AppBarModelStatus
              onOpenSettings={() => {
                close();
                onSettings();
              }}
            />
            <button
              type="button"
              className={secondaryButton}
              onClick={() => {
                close();
                onSettings();
              }}
            >
              設定・モデル管理
            </button>
          </>
        )}
      </dialog>
    </>
  );
}
