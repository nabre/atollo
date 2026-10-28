// How shut in the projects' programs are (D-167): the status bar's item,
// a warning while the projects run without the VM, and the box that says
// why the VM is off and what makes it work, one step per reason, with
// Atollo's sandboxes and the room they take (D-169).
import { useState } from "react";

import {
  componentsInstall,
  isolationRun,
  isolationStatus,
  vmRemove,
  vmsList,
  type IsolationBlocker,
  type IsolationFix,
  type IsolationStatus,
  type Job,
  type VmInfo,
  type VmList,
} from "../ipc";
import { formatLargeBytes } from "../shared/format";
import { useAction, useCopy, useLoad } from "../shared/hooks";
import { useSetupEnded } from "../components/setup";
import { usePoll } from "../shared/poll";
import { isolationPoll } from "../shared/polls";
import { Button, Codicon, ConfirmDialog } from "../workbench";
import { bytesPercent, overallPercent, stepText } from "./jobs";
import { t } from "../i18n";

/** What a blocker means and what the user does about it: a sentence, the
 * command to run when there is one (`run` when Atollo launches it in its
 * terminal: only `sbx`'s own), `admin` when that command asks for an
 * administrator or a new sign-in (shown to be copied, never launched: zero
 * admin, D-178), and `install` when the kit's Docker Sandboxes is what is
 * missing (a button installs it). */
interface Explained {
  why: string;
  fix: string;
  command?: string;
  run?: IsolationFix;
  admin?: boolean;
  install?: boolean;
}

/** A path AppArmor reads as itself: absolute, with none of the characters
 * that would end the quoted name, start a rule or a glob. */
export function plainProfilePath(path: string): boolean {
  return path.startsWith("/") && !/["\\\n\r\t*?[\]{}^,#@$]/.test(path);
}

/** The AppArmor profile that lets the kit's microVM shim create user
 * namespaces (the rule Ubuntu restricts), as Docker's package does for its
 * own; `null` when the path is not a plain one: a command run with `sudo`
 * never carries text that could add rules. */
export function apparmorCommand(shim: string): string | null {
  if (!plainProfilePath(shim)) return null;
  const file = "/etc/apparmor.d/atollo-sbx-shim";
  return [
    `sudo tee ${file} >/dev/null <<'EOF'`,
    "abi <abi/3.0>,",
    "include <tunables/global>",
    `profile atollo-sbx-shim "${shim}" flags=(unconfined) {`,
    "  userns,",
    "}",
    "EOF",
    `sudo apparmor_parser -r ${file}`,
  ].join("\n");
}

export function explain(b: IsolationBlocker): Explained {
  switch (b.code) {
    case "old_windows":
      return {
        why: t("Windows build {0} non è Windows 11: Docker Sandboxes funziona solo su Windows 11.", String(b.build)),
        fix: t("Aggiorna il PC a Windows 11 (64 bit, processore Intel o AMD)."),
      };
    case "old_macos":
      return {
        why: t("macOS {0}: Docker Sandboxes richiede macOS 14 Sonoma o successivo.", b.version),
        fix: t("Aggiorna macOS da Impostazioni di Sistema › Generali › Aggiornamento Software."),
      };
    case "distribution":
      return {
        why: t("{0}: Docker Sandboxes supporta solo Ubuntu 24.04 o successivo (non le derivate).", b.name),
        fix: t("Su questa distribuzione la VM non è disponibile: i progetti restano sul PC."),
      };
    case "hypervisor_off":
      return {
        why: t("Windows Hypervisor Platform non è attivo (o il PC non è stato riavviato dopo averlo attivato)."),
        fix: t("In PowerShell come amministratore, una volta sola, poi riavvia il PC:"),
        command: "Enable-WindowsOptionalFeature -Online -FeatureName HypervisorPlatform -All",
        admin: true,
      };
    case "kvm_missing":
      return {
        why: t("/dev/kvm non c'è: la virtualizzazione è spenta nel firmware o manca il modulo KVM."),
        fix: t("Attiva la virtualizzazione (Intel VT-x o AMD-V) nel BIOS/UEFI e riavvia il PC."),
      };
    case "kvm_denied":
      return {
        why: t("Il tuo utente non può usare /dev/kvm: non è nel gruppo kvm."),
        fix: t("In un terminale del PC, poi esci e rientra nella sessione:"),
        command: "sudo usermod -aG kvm $USER",
        admin: true,
      };
    case "apparmor": {
      const command = apparmorCommand(b.shim);
      return command
        ? {
            why: t("AppArmor non lascia partire la microVM di Docker Sandboxes del kit: le manca il profilo."),
            fix: t("In un terminale del PC, una volta (di nuovo se il kit cambia cartella):"),
            command,
            admin: true,
          }
        : {
            why: t("AppArmor non lascia partire la microVM di Docker Sandboxes del kit: le manca il profilo."),
            fix: t(
              "La cartella del kit ha caratteri che un profilo AppArmor non accetta (virgolette, *, ?, [ ], { }, virgole…): sposta il kit in una cartella con un nome semplice.",
            ),
          };
    }
    case "sbx_missing":
      return {
        why: b.failed
          ? t("Docker Sandboxes (sbx) non è nel kit: l'ultima installazione non è riuscita ({0}).", b.failed)
          : t("Docker Sandboxes (sbx) non è nel kit. Docker Desktop non serve."),
        fix: t(
          "Installalo nel kit (circa 470 MB in data, portabile come gli altri programmi); poi accedi con «sbx login» nel terminale di Atollo.",
        ),
        install: true,
      };
    case "sbx_signed_out":
      return {
        why: t("sbx non ha accesso a Docker: serve un account Docker (gratuito)."),
        fix: t("Nel terminale di Atollo (si apre il browser per l'accesso):"),
        command: "sbx login",
        run: "login",
      };
    case "sbx_failed":
      return {
        why: t("sbx ha risposto con un errore: {0}", b.message),
        fix: t("Prova il comando nel terminale di Atollo per vedere il problema:"),
        command: "sbx ls",
        run: "ls",
      };
    case "foreign_daemon":
      return {
        why: t(
          "Sul PC è già attivo un altro Docker Sandboxes (quello del PC o di un altro kit): le VM del kit finirebbero nei suoi dati, fuori dal kit.",
        ),
        fix: t("Fermalo, se non ti serve, poi premi «Ricontrolla»; le sue sandbox restano dove sono:"),
        command: "sbx daemon stop",
      };
    case "kit_disk":
      return {
        why: t(
          "Il kit è su un disco {0}: i dischi delle VM (fino a 30 GB ciascuna, che su NTFS occupano solo quello che si scrive) non ci stanno o lo riempirebbero.",
          b.fs,
        ),
        fix: t("Sposta il kit su un disco NTFS (o APFS, ext4 su macOS e Linux), poi riaprilo da lì."),
      };
  }
}

/** A command with its button: «Lancia» opens it in the app's terminal
 * when it is one of `sbx`'s (`run`), then `onLaunched` (the box closes,
 * the terminal is in front); «Copia» for one of the PC's, with a note when
 * it asks for an administrator (`admin`): never launched by Atollo. */
function CommandLine({
  command,
  run,
  admin,
  onLaunched,
}: {
  command: string;
  run?: IsolationFix | undefined;
  admin?: boolean | undefined;
  onLaunched: () => void;
}) {
  const { copied, copy } = useCopy(2000);
  const [error, setError] = useState<string | null>(null);
  return (
    <div className="mt-vsc-size40">
      <div className="flex items-start gap-vsc-size40">
        <code className="flex-1 rounded-vsc-small bg-vsc-input px-vsc-size60 py-vsc-size20 font-mono text-[12px] break-all whitespace-pre-wrap text-vsc-fg select-text">
          {command}
        </code>
        {run ? (
          <Button
            small
            variant="primary"
            onClick={() => {
              setError(null);
              isolationRun(run).then(onLaunched, (e: unknown) => {
                setError(String(e));
              });
            }}
          >
            {t("Lancia")}
          </Button>
        ) : (
          <Button
            small
            onClick={() => {
              copy(command);
            }}
          >
            {copied ? t("Copiato") : t("Copia")}
          </Button>
        )}
      </div>
      {admin && (
        <p className="mt-vsc-size20 text-vsc-desc">
          {t(
            "Serve un amministratore: Atollo non lo esegue mai, lo copi e lo esegui tu. Dopo può servire riavviare il PC o rientrare nella sessione; Atollo ricontrolla da solo.",
          )}
        </p>
      )}
      {error && <p className="mt-vsc-size20 text-vsc-error">{error}</p>}
    </div>
  );
}

/** "Installa nel kit": the kit's Docker Sandboxes, as on the Tools page;
 * the progress is in the status bar. */
function InstallButton() {
  const [state, setState] = useState<"idle" | "busy" | "started">("idle");
  const [error, setError] = useState<string | null>(null);
  return (
    <div className="mt-vsc-size40 flex items-center gap-vsc-size80">
      <Button
        small
        variant="primary"
        busy={state === "busy"}
        disabled={state === "started"}
        onClick={() => {
          setState("busy");
          setError(null);
          componentsInstall("sandboxes").then(
            () => {
              setState("started");
            },
            (e: unknown) => {
              setState("idle");
              setError(String(e));
            },
          );
        }}
      >
        {t("Installa nel kit")}
      </Button>
      {state === "started" && <span>{t("Installazione avviata: la vedi nella barra di stato.")}</span>}
      {error && <span className="text-vsc-error">{error}</span>}
    </div>
  );
}

/** Why the VM is off and what makes it work. */
export function IsolationDialog({
  status,
  checking,
  onCheck,
  onClose,
}: {
  status: IsolationStatus;
  checking: boolean;
  onCheck: () => void;
  onClose: () => void;
}) {
  const vm = status.level === "vm";
  return (
    <ConfirmDialog
      title={vm ? t("Progetti nella VM") : t("Lavori senza VM")}
      cancelLabel={t("Chiudi")}
      onCancel={onClose}
      actions={
        <Button variant="primary" busy={checking} onClick={onCheck}>
          {t("Ricontrolla")}
        </Button>
      }
    >
      {vm ? (
        <p>
          {t(
            "Ogni progetto gira in una microVM di Docker Sandboxes: terminale, azioni, server, terminali dell'editor e agenti vedono solo la cartella del progetto. I terminali del kit restano sul PC, senza gli agenti.",
          )}
        </p>
      ) : (
        <>
          <p className="text-vsc-fg">
            {t(
              "Senza VM i progetti, il loro terminale e le azioni (npm run dev, i siti, i server condivisi) girano sul PC con il tuo utente, senza gli agenti: possono leggere i tuoi file e usare la rete, il kit ne filtra solo l'ambiente.",
            )}
          </p>
          <p className="text-vsc-fg">
            {t(
              "I terminali, i task e gli agenti dell'editor e gli agenti del progetto partono solo nella VM: senza, non si aprono. Atollo ricontrolla da solo ogni minuto.",
            )}
          </p>
          <p className="font-vsc-semibold text-vsc-fg">{t("Perché la VM non è attiva")}</p>
          <ol className="list-decimal space-y-vsc-size80 pl-5">
            {status.blockers.map((b) => {
              const e = explain(b);
              return (
                <li key={b.code}>
                  <p className="text-vsc-fg">{e.why}</p>
                  <p>{e.fix}</p>
                  {e.command && <CommandLine command={e.command} run={e.run} admin={e.admin} onLaunched={onClose} />}
                  {e.install && <InstallButton />}
                </li>
              );
            })}
          </ol>
        </>
      )}
      {status.sbx !== null && <VmSection />}
    </ConfirmDialog>
  );
}

/** Whose sandbox it is, as the list says it (the kit's terminals have none
 * since D-178: they run on the PC; the tools sandbox builds the kit's tools
 * for the VM's Linux, D-180). */
export function vmLabel(v: VmInfo): string {
  if (v.owner?.kind === "project") return t("Progetto {0}", v.owner.project);
  if (v.owner?.kind === "folder") return t("Cartella {0}", v.owner.dir);
  if (v.owner?.kind === "tools") return t("Strumenti del kit per il Linux della VM");
  return v.name;
}

/** Atollo's sandboxes (D-169): whose each is, whether it runs, the room
 * their state takes in the kit, and «Rimuovi» after a second click, since
 * it deletes the sandbox's disk. */
export function VmSection() {
  const { data, error, reload } = useLoad<VmList | null>(vmsList, null, { onError: "show" });
  const { busy, error: failed, run } = useAction(reload);
  const [asked, setAsked] = useState<string | null>(null);
  if (!data) return error ? <p className="text-vsc-error">{error}</p> : null;
  return (
    <section aria-label={t("Sandbox delle VM")} className="space-y-vsc-size40">
      <p className="font-vsc-semibold text-vsc-fg">{t("Sandbox delle VM")}</p>
      {data.disk !== null && <p>{t("Spazio occupato nel kit: {0}", formatLargeBytes(data.disk))}</p>}
      {data.vms.length === 0 ? (
        <p>{t("Nessuna sandbox.")}</p>
      ) : (
        <ul className="space-y-vsc-size40">
          {data.vms.map((v) => (
            <li key={v.name} className="flex items-center gap-vsc-size80">
              <span className="min-w-0 flex-1 truncate text-vsc-fg" title={v.name}>
                {vmLabel(v)}
              </span>
              <span className="text-vsc-desc">{v.status === "running" ? t("accesa") : t("spenta")}</span>
              {asked === v.name ? (
                <>
                  <Button
                    small
                    variant="primary"
                    onClick={() => {
                      setAsked(null);
                      run(v.name, () => vmRemove(v.name));
                    }}
                  >
                    {t("Rimuovi davvero")}
                  </Button>
                  <Button
                    small
                    onClick={() => {
                      setAsked(null);
                    }}
                  >
                    {t("Annulla")}
                  </Button>
                </>
              ) : (
                <Button
                  small
                  busy={busy === v.name}
                  onClick={() => {
                    setAsked(v.name);
                  }}
                >
                  {t("Rimuovi")}
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}
      {asked !== null && (
        <p>
          {t(
            "Si cancella il disco della sandbox: la sua home, /tmp e le immagini Docker scaricate. I file e il database del progetto restano.",
          )}
        </p>
      )}
      {failed && <p className="text-vsc-error">{failed}</p>}
    </section>
  );
}

/** The status, read at start and every minute (`sbx` is asked at most once
 * a minute), and asked again on "Ricontrolla" and when an installation
 * ends (Docker Sandboxes may have arrived). The
 * poll is the one of the Web Servers page too (shared/polls), which reads
 * it more often while open. */
export function useIsolation() {
  const { data, setData } = usePoll<IsolationStatus, IsolationStatus | null>(isolationPoll, null, {
    onError: "keep",
    every: 60_000,
  });
  const [checking, setChecking] = useState(false);
  const check = () => {
    setChecking(true);
    isolationStatus(true).then(
      (st) => {
        setData(st);
        setChecking(false);
      },
      () => {
        setChecking(false);
      },
    );
  };
  useSetupEnded(() => {
    isolationStatus(true).then(setData, () => {});
  });
  return { status: data, checking, check };
}

/** The blockers the user can remove on this PC (a sign-in, the
 * hypervisor, the kit's `sbx`, a group, a profile, another daemon); the
 * others need another system, its firmware or another disk. */
const FIXABLE: ReadonlySet<IsolationBlocker["code"]> = new Set([
  "hypervisor_off",
  "kvm_denied",
  "apparmor",
  "sbx_missing",
  "sbx_signed_out",
  "foreign_daemon",
]);

/** The VM's pill in the status bar (LEVELS.md §8.6): its tone, its text,
 * the percent of a preparation, what it says in full (tooltip and
 * accessible name) and what to do. */
export interface VmPill {
  tone: "ready" | "busy" | "fixable" | "off" | "error";
  text: string;
  percent: number | null;
  label: string;
  todo: string;
}

/** The pill of `status` with the jobs: a VM being prepared is «VM…» with
 * its percent; the last preparation failed (none after it ended well) is
 * «VM ⚠»; else «VM» when on, «Senza VM» orange when the user can fix why,
 * grey when this PC cannot have it, red when `sbx` answers with an error. */
export function vmPill(status: IsolationStatus, jobs: Job[]): VmPill {
  const prep = jobs.filter((j) => j.kind === "vm-prepare");
  const busy = prep.find((j) => j.state === "running");
  if (busy) {
    const percent = bytesPercent(busy) ?? overallPercent(busy);
    return {
      tone: "busy",
      text: percent === null ? t("VM…") : `${t("VM…")} ${String(percent)}%`,
      percent,
      label: t("VM in preparazione: {0}", `${busy.title} · ${stepText(busy)}`),
      todo: t("Clic: i dettagli della VM; l'avanzamento è in Attività."),
    };
  }
  const last = prep.filter((j) => j.state === "done" || j.state === "error").at(-1);
  if (status.level === "vm" && last?.state === "error") {
    return {
      tone: "error",
      text: t("VM ⚠"),
      percent: null,
      label: t("La VM non si è preparata: {0}", last.error ?? last.title),
      todo: t("Clic: i dettagli della VM; riprova dal progetto."),
    };
  }
  if (status.level === "vm") {
    return {
      tone: "ready",
      text: t("VM"),
      percent: null,
      label: t("Progetti nella VM"),
      todo: t("Clic: i dettagli della VM"),
    };
  }
  if (status.blockers.some((b) => b.code === "sbx_failed")) {
    return {
      tone: "error",
      text: t("Senza VM"),
      percent: null,
      label: t("Senza VM: Docker Sandboxes ha risposto con un errore"),
      todo: t("Clic: cosa è successo e come riprovare"),
    };
  }
  const fixable = status.blockers.some((b) => FIXABLE.has(b.code));
  return {
    tone: fixable ? "fixable" : "off",
    text: t("Senza VM"),
    percent: null,
    label: fixable
      ? t("Senza VM, si può attivare: progetti e azioni sul PC senza agenti")
      : t("Senza VM: questo PC non la può avere; progetti e azioni sul PC senza agenti"),
    todo: fixable ? t("Clic: cosa serve per attivare la VM") : t("Clic: perché questo PC non può avere la VM"),
  };
}

const PILL: Record<VmPill["tone"], string> = {
  ready: "bg-vsc-pill-ready",
  busy: "bg-vsc-pill-busy animate-pulse",
  fixable: "bg-vsc-pill-fixable",
  off: "bg-vsc-pill-off",
  error: "bg-vsc-pill-error",
};

/** The pill itself: the VM's icon and its text on the tone's color, the
 * text in the editor's background (readable in both themes). The state is
 * in the text and the accessible name, not in the color alone. */
export function VmPillBadge({ pill }: { pill: VmPill }) {
  return (
    <span
      data-tone={pill.tone}
      className={`inline-flex h-[16px] items-center gap-vsc-size20 rounded-vsc-circle px-vsc-size60 font-semibold text-vsc-bg ${PILL[pill.tone]}`}
    >
      <Codicon name="vm" size={12} />
      <span className="tabular-nums">{pill.text}</span>
    </span>
  );
}

/** A sandbox's state on the VM pill's colours (the project's «VM del
 * progetto», LEVELS.md §8.5): green «accesa», blue and pulsing while an
 * action works on it, grey «spenta» or «non c'è». The state is in the
 * text, not in the colour alone. */
export function SandboxPill({ state, busy = false }: { state: "running" | "stopped" | "missing"; busy?: boolean }) {
  const tone: VmPill["tone"] = busy ? "busy" : state === "running" ? "ready" : "off";
  const text = busy
    ? t("in corso…")
    : state === "running"
      ? t("accesa")
      : state === "stopped"
        ? t("spenta")
        : t("non c'è");
  return (
    <span
      data-tone={tone}
      className={`inline-flex h-[16px] items-center gap-vsc-size20 rounded-vsc-circle px-vsc-size60 font-semibold text-vsc-bg ${PILL[tone]}`}
    >
      <Codicon name="vm" size={12} />
      <span>{text}</span>
    </span>
  );
}
