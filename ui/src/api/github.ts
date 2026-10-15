// The kit's GitHub account (tauri/src/github/): one sign-in for this
// page, the kit's Git (editor, consoles, agents) and GitHub CLI.
// Rust: tauri/src/commands/github.rs.

import { invoke } from "@tauri-apps/api/core";
import { act, query, subscribe } from "./core";

export interface GithubStatus {
  /** GitHub CLI is in the kit. */
  installed: boolean;
  signedIn: boolean;
  login: string | null;
  /** A sign-in is waiting for the user on github.com. */
  signingIn: boolean;
  /** The one-time code to type at github.com/login/device. */
  code: string | null;
  /** Why the last sign-in failed. */
  error: string | null;
}

export interface GithubRepo {
  fullName: string;
  name: string;
  owner: string;
  private: boolean;
  fork: boolean;
  archived: boolean;
  description: string;
  language: string;
  stars: number;
  pushedAt: string | null;
  htmlUrl: string;
  /** The account administers it: rename and delete. */
  admin: boolean;
  /** The project in projects\ that is its clone. */
  local: string | null;
}

export interface GithubRepos {
  login: string;
  /** The account may delete repositories (null: the token does not say). */
  canDelete: boolean | null;
  repos: GithubRepo[];
}

const SAMPLE_GITHUB: GithubStatus = {
  installed: true,
  signedIn: true,
  login: "octocat",
  signingIn: false,
  code: null,
  error: null,
};

function sampleRepo(owner: string, name: string, extra: Partial<GithubRepo>): GithubRepo {
  return {
    fullName: `${owner}/${name}`,
    name,
    owner,
    private: false,
    fork: false,
    archived: false,
    description: "",
    language: "",
    stars: 0,
    pushedAt: "2026-10-01T10:00:00Z",
    htmlUrl: `https://github.com/${owner}/${name}`,
    admin: owner === "octocat",
    local: null,
    ...extra,
  };
}

const SAMPLE_REPOS: GithubRepos = {
  login: "octocat",
  canDelete: false,
  repos: [
    sampleRepo("octocat", "sito-vetrina", {
      language: "HTML",
      local: "sito-vetrina",
      description: "Il sito del negozio",
    }),
    sampleRepo("octocat", "api-python", { language: "Python", private: true, stars: 3 }),
    sampleRepo("octocat", "dotfiles", { fork: true }),
    sampleRepo("acme", "gestionale", { language: "TypeScript", private: true, description: "Gestionale interno" }),
  ],
};

export function githubStatus(): Promise<GithubStatus> {
  return query<GithubStatus>("github_status", SAMPLE_GITHUB);
}

/** Starts the sign-in; answers with the one-time code. `deleteScope`: also the
 * permission to delete repositories. */
export function githubSignIn(deleteScope: boolean): Promise<GithubStatus> {
  return query<GithubStatus>(
    "github_sign_in",
    { ...SAMPLE_GITHUB, signedIn: false, signingIn: true, code: "1A2B-3C4D" },
    { delete: deleteScope },
  );
}

export function githubSignInCancel(): Promise<void> {
  return act("github_sign_in_cancel");
}

export function githubSignOut(): Promise<void> {
  return act("github_sign_out");
}

export function githubRepos(): Promise<GithubRepos> {
  return query<GithubRepos>("github_repos", SAMPLE_REPOS);
}

/** Clones a repository of the account into projects\; returns the project. */
export function githubClone(fullName: string): Promise<string> {
  return query<string>("github_clone", fullName.split("/")[1] ?? "", { fullName });
}

export function githubRename(fullName: string, name: string): Promise<{ repo: GithubRepo; local: string | null }> {
  return invoke<{ repo: GithubRepo; local: string | null }>("github_rename", { fullName, name });
}

export function githubDelete(fullName: string): Promise<"deleted" | "needsPermission"> {
  return query<"deleted" | "needsPermission">("github_delete", "needsPermission", { fullName });
}

/** Opens a repository on github.com (`null`: the page for the sign-in code). */
export function githubOpen(fullName: string | null): Promise<void> {
  return act("github_open", { fullName });
}

/** The sign-in got its code or ended. */
export function onGithub(handler: (s: GithubStatus) => void): Promise<() => void> {
  return subscribe<GithubStatus>("github", handler);
}
