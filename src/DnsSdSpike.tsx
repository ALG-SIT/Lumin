import type {
  AdvertiseHandle,
  ServiceAnnouncement,
} from "@momics/dns-sd-tauri";
import { advertise, browse, close } from "@momics/dns-sd-tauri";
import { useCallback, useEffect, useRef, useState } from "react";

type Mode = "idle" | "teacher" | "student";

interface LogEntry {
  time: string;
  message: string;
}

export function DnsSdSpike() {
  const [mode, setMode] = useState<Mode>("idle");
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [discovered, setDiscovered] = useState<ServiceAnnouncement[]>([]);
  const advertiseHandle = useRef<AdvertiseHandle | null>(null);
  const browseAbort = useRef<AbortController | null>(null);

  const log = useCallback((msg: string) => {
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [...prev, { time, message: msg }]);
    console.log(`[DNS-SD Spike ${time}]`, msg);
  }, []);

  const startTeacher = async () => {
    try {
      log(
        "Starting teacher mode — advertising _lumin-class._tcp on port 9876...",
      );
      const handle = await advertise({
        service: {
          type: "lumin-class",
          protocol: "tcp",
          name: "Lumin Classroom Spike",
          port: 9876,
          txt: { code_required: "true", version: "spike-0.1" },
        },
      });
      advertiseHandle.current = handle;
      setMode("teacher");
      log(`Advertising as "${handle.name}" on port 9876`);
    } catch (err) {
      log(`ERROR starting teacher: ${err}`);
    }
  };

  const startStudent = async () => {
    try {
      log("Starting student mode — browsing for _lumin-class._tcp...");
      setMode("student");
      setDiscovered([]);

      const controller = new AbortController();
      browseAbort.current = controller;

      // Run browse in background
      (async () => {
        try {
          for await (const svc of browse({
            service: { type: "lumin-class", protocol: "tcp" },
            signal: controller.signal,
          })) {
            log(
              `[${svc.kind}] ${svc.name} → ${svc.host ?? "?"}:${svc.port ?? "?"}`,
            );
            if (svc.kind === "resolved" || svc.kind === "updated") {
              setDiscovered((prev) => {
                const existing = prev.findIndex((p) => p.name === svc.name);
                if (existing >= 0) {
                  const next = [...prev];
                  next[existing] = svc;
                  return next;
                }
                return [...prev, svc];
              });
            }
            if (svc.kind === "removed") {
              setDiscovered((prev) => prev.filter((p) => p.name !== svc.name));
            }
          }
        } catch (err) {
          if ((err as Error).name !== "AbortError") {
            log(`Browse error: ${err}`);
          }
        }
      })();

      log("Browse started. Waiting for services...");
    } catch (err) {
      log(`ERROR starting student: ${err}`);
    }
  };

  const stop = async () => {
    log("Stopping...");
    if (advertiseHandle.current) {
      await advertiseHandle.current.stop();
      advertiseHandle.current = null;
      log("Advertisement stopped");
    }
    if (browseAbort.current) {
      browseAbort.current.abort();
      browseAbort.current = null;
      log("Browse stopped");
    }
    await close();
    setMode("idle");
    setDiscovered([]);
    log("Closed DNS-SD adapter");
  };

  useEffect(() => {
    return () => {
      // Cleanup on unmount
      if (advertiseHandle.current)
        advertiseHandle.current.stop().catch(() => {});
      if (browseAbort.current) browseAbort.current.abort();
      close().catch(() => {});
    };
  }, []);

  return (
    <div style={{ padding: "2rem", fontFamily: "monospace", maxWidth: 800 }}>
      <h1>DNS-SD Discovery Spike</h1>
      <p>
        Testing @momics/dns-sd-tauri — service discovery for Lumin classroom
      </p>

      <div style={{ display: "flex", gap: "1rem", marginBottom: "1rem" }}>
        <button type="button" onClick={startTeacher} disabled={mode !== "idle"}>
          Start Teacher (Advertise)
        </button>
        <button type="button" onClick={startStudent} disabled={mode !== "idle"}>
          Start Student (Browse)
        </button>
        <button type="button" onClick={stop} disabled={mode === "idle"}>
          Stop All
        </button>
      </div>

      <div style={{ marginBottom: "1rem" }}>
        <strong>Mode:</strong> {mode}
      </div>

      {discovered.length > 0 && (
        <div style={{ marginBottom: "1rem" }}>
          <h3>Discovered Services ({discovered.length})</h3>
          <ul>
            {discovered.map((svc, i) => (
              /* biome-ignore lint/suspicious/noArrayIndexKey: 同名サービスが衝突し得るため index を使用 */
              <li key={i}>
                <strong>{svc.name}</strong> — {svc.host}:{svc.port}
                {svc.txt && (
                  <span>
                    {" "}
                    TXT:{" "}
                    {JSON.stringify(
                      Object.fromEntries(
                        Object.entries(svc.txt).map(([k, v]) => [
                          k,
                          v instanceof Uint8Array
                            ? new TextDecoder().decode(v)
                            : String(v),
                        ]),
                      ),
                    )}
                  </span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      <div>
        <h3>Log ({logs.length} entries)</h3>
        <div
          style={{
            background: "#1a1a2e",
            color: "#e0e0e0",
            padding: "1rem",
            borderRadius: 4,
            maxHeight: 400,
            overflow: "auto",
            fontSize: 12,
          }}
        >
          {logs.map((entry, i) => (
            /* biome-ignore lint/suspicious/noArrayIndexKey: 追記のみのログリストでIDを持たないため */
            <div key={i}>
              [{entry.time}] {entry.message}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
