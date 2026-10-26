// The three levels in the pages (D-179, docs/app/LEVELS.md §8.3): the text
// of a terminal's label, and what the user does about a refused command
// (its `fix`, LEVELS.md §4.1).
import type { Fix, Level, LevelLabel } from "../ipc/levels";
import { t } from "../i18n";
import { Link } from "../workbench";

/** The text of a terminal's label (8.3): «PC» at level 1, «PC · senza VM»
 * for project code on the PC, «VM progetto», «Agente · VM». */
export function labelText(label: LevelLabel, level: Level): string {
  switch (label) {
    case "pc":
      return level === "project" ? t("PC · senza VM") : t("PC");
    case "projectVm":
      return t("VM progetto");
    case "agentVm":
      return t("Agente · VM");
  }
}

/** The event that opens the "No VM" notice of the status bar (the fixes
 * `open_vm_notice` and `restart_daemon`: its «Ricontrolla» is there). */
export const OPEN_VM_NOTICE = "atollo:open-vm-notice";

export function openVmNotice(): void {
  window.dispatchEvent(new Event(OPEN_VM_NOTICE));
}

/** The action of a refusal's fix, when the app has one: the VM's notice.
 * The other fixes are said by the refusal's text (open it from the
 * project, move the project to a local disk, start the site from its
 * project). */
export function FixAction({ fix }: { fix: Fix | null | undefined }) {
  if (!hasAction(fix)) return null;
  return <Link onClick={openVmNotice}>{t("Apri l'avviso VM")}</Link>;
}

/** True when the app has an action for `fix` ([`FixAction`]). */
export function hasAction(fix: Fix | null | undefined): boolean {
  return fix === "open_vm_notice" || fix === "restart_daemon";
}
