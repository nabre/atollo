// The kit's key folders, as the Guide names them: the app opens them, the
// site lists them.
import { tr } from "./core/lang";

export interface KitFolder {
  /** What the app's folderOpen takes. */
  id: "projects" | "agents" | "db" | "backup";
  label: string;
  path: string;
}

export function kitFolders(): KitFolder[] {
  return [
    {
      id: "projects",
      label: tr("Cartella progetti", "Projects folder", "Dossier des projets", "Projektordner"),
      path: "projects\\",
    },
    {
      id: "agents",
      label: tr("Agenti condivisi", "Shared agents", "Agents partagés", "Geteilte Agenten"),
      path: "data\\agents\\",
    },
    {
      id: "db",
      label: tr("Dati database", "Database data", "Données de bases de données", "Datenbankdaten"),
      path: "data\\db\\",
    },
    {
      id: "backup",
      label: tr("Backup database", "Database backups", "Sauvegardes de bases de données", "Datenbank-Backups"),
      path: tr(
        "data\\projects\\<progetto>\\private\\backups\\",
        "data\\projects\\<project>\\private\\backups\\",
        "data\\projects\\<projet>\\private\\backups\\",
        "data\\projects\\<projekt>\\private\\backups\\",
      ),
    },
  ];
}
