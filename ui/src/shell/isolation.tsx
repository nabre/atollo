// How shut in the projects' programs are (D-167): the status bar's item,
// a warning while the projects run without the VM, and the box that says
// why the VM is off and what makes it work, one step per reason.
import { useState } from "react";

import { isolationStatus, type IsolationBlocker, type IsolationStatus } from "../ipc";
import { useCopy, useLoad } from "../shared/hooks";
import { Button, ConfirmDialog } from "../workbench";
import { t } from "../i18n";

/** The PC's system, from the kit's platform (`win32-x64`, `darwin-arm64`). */
export type System = "windows" | "macos" | "linux";

export function systemOf(platform: string): System {
  if (platform.startsWith("win32")) return "windows";
  if (platform.startsWith("darwin")) return "macos";
  return "linux";
}

/** What a blocker means and what the user does about it: a sentence, and
 * the command to run when there is one. */
interface Explained {
  why: string;
  fix: string;
  command?: string;
}

const INSTALL: Record<System, string> = {
  windows: "winget install -h Docker.sbx",
  macos: "brew install docker/tap/sbx",
  linux: "curl -fsSL https://get.docker.com | sudo REPO_ONLY=1 sh && sudo apt install docker-sbx",
};

export function explain(b: IsolationBlocker, system: System): Explained {
  switch (b.code) {
    case "old_windows":
      return {
        why: t("Windows build {0} non è Windows 11: Docker Sandboxes funziona solo su Windows 11.", String(b.build)),
        fix: t("Aggiorna il PC a Windows 11 (64 bit, processore Intel o AMD)."),
      };
    case "processor":
      return system === "macos"
        ? {
            why: t("Docker Sandboxes richiede un Mac con Apple silicon (M1 o successivo)."),
            fix: t("Su un Mac Intel la VM non è disponibile: i progetti restano sul PC."),
          }
        : {
            why: t("Docker Sandboxes non funziona su Windows per processori ARM."),
            fix: t("Su questo PC la VM non è disponibile: i progetti restano sul PC."),
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
      };
    case "sbx_missing":
      return {
        why: t("Docker Sandboxes (sbx) non è installato su questo PC. Docker Desktop non serve."),
        fix: t("Installalo in un terminale del PC, poi accedi con «sbx login»:"),
        command: INSTALL[system],
      };
    case "sbx_signed_out":
      return {
        why: t("sbx è installato ma non ha accesso a Docker: serve un account Docker (gratuito)."),
        fix: t("In un terminale del PC (si apre il browser per l'accesso):"),
        command: "sbx login",
      };
    case "sbx_failed":
      return {
        why: t("sbx ha risposto con un errore: {0}", b.message),
        fix: t("Prova il comando in un terminale del PC per vedere il problema:"),
        command: "sbx ls",
      };
    case "pending":
      return {
        why: t("Il PC ha tutto, ma questa versione di Atollo non avvia ancora i progetti nella VM."),
        fix: t("Nessuna azione: la VM si attiva da sola con un aggiornamento di Atollo."),
      };
  }
}

/** A command with its "Copia" button. */
function CommandLine({ command }: { command: string }) {
  const { copied, copy } = useCopy(2000);
  return (
    <div className="mt-vsc-size40 flex items-start gap-vsc-size40">
      <code className="flex-1 rounded-vsc-small bg-vsc-input px-vsc-size60 py-vsc-size20 font-mono text-[12px] break-all text-vsc-fg select-text">
        {command}
      </code>
      <Button
        small
        onClick={() => {
          copy(command);
        }}
      >
        {copied ? t("Copiato") : t("Copia")}
      </Button>
    </div>
  );
}

/** Why the VM is off and what makes it work. */
export function IsolationDialog({
  status,
  system,
  checking,
  onCheck,
  onClose,
}: {
  status: IsolationStatus;
  system: System;
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
            "Ogni progetto gira in una microVM di Docker Sandboxes: terminale, azioni, server e agenti vedono solo la cartella del progetto.",
          )}
        </p>
      ) : (
        <>
          <p className="text-vsc-fg">
            {t(
              "I programmi dei progetti (terminale, azioni, server, agenti) girano sul PC con il tuo utente: possono leggere i tuoi file e usare la rete. Il kit ne filtra solo l'ambiente.",
            )}
          </p>
          <p className="font-vsc-semibold text-vsc-fg">{t("Perché la VM non è attiva")}</p>
          <ol className="list-decimal space-y-vsc-size80 pl-5">
            {status.blockers.map((b) => {
              const e = explain(b, system);
              return (
                <li key={b.code}>
                  <p className="text-vsc-fg">{e.why}</p>
                  <p>{e.fix}</p>
                  {e.command && <CommandLine command={e.command} />}
                </li>
              );
            })}
          </ol>
        </>
      )}
    </ConfirmDialog>
  );
}

/** The status, read at start and every minute (`sbx` is asked at most once
 * a minute), and asked again on "Ricontrolla". */
export function useIsolation() {
  const { data, setData } = useLoad<IsolationStatus | null>(isolationStatus, null, {
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
  return { status: data, checking, check };
}
