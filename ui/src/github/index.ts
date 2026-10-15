// GitHub page: the kit's GitHub account and its repositories (port of
// nabre/atollo setup/extensions/github-repos). One sign-in for everything:
// this page, the kit's Git (editor, consoles, agents) and GitHub CLI, saved
// in data\gh (tauri/src/github.rs, DECISIONS.md D-70).
export { GithubAccount, useGithub } from "./account";
export { GithubRepositories } from "./repos";
