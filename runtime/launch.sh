#!/bin/sh
# Atollo: the request client of the levels for sh (written by Atollo.exe to
# data/kit/launch.sh; edits are replaced at the next start). DECISIONS.md
# D-179, docs/app/LEVELS.md section 4.3. The same as launch.ps1, for a
# system or a shell without the kit's PowerShell:
#   sh launch.sh editorTask [-c] <line>          (a task: VSCodium adds -c)
#   sh launch.sh editorAgentProfile <agent>      (an agent's profile)
#   sh launch.sh forwarded <agent> [args...]     (claude, codex, pi, agy typed)
#
# It decides nothing: it writes a request in data/app/levels/requests/ with
# the origin and the current folder, waits for Atollo's reply (written for
# sh too: variables and `set --`), and runs what it says in this terminal
# (stdio passed through), exiting with its code. A refusal prints why and
# what to do, and exits 1.

umask 077
kit=$(cd "$(dirname "$0")" && pwd)
requests="$kit/../app/levels/requests"

say() { printf '%s\n' "$1" >&2; }

# $1 as the inside of a JSON string.
json() {
    ATOLLO_JSON=$1 awk 'BEGIN {
        s = ENVIRON["ATOLLO_JSON"]; out = ""; n = length(s)
        for (i = 1; i <= n; i++) {
            c = substr(s, i, 1)
            if (c == "\\") c = "\\\\"
            else if (c == "\"") c = "\\\""
            else if (c == "\n") c = "\\n"
            else if (c == "\r") c = "\\r"
            else if (c == "\t") c = "\\t"
            out = out c
        }
        printf "%s", out
    }'
}

origin=$1
[ $# -gt 0 ] && shift
case $origin in
    editorTask)
        [ "$1" = "-c" ] && shift
        if [ $# -lt 1 ] || [ -z "$1" ]; then
            say "launch.sh: editorTask [-c] <line>"
            exit 2
        fi
        what="{\"kind\":\"editorTask\",\"command\":\"$(json "$1")\"}"
        shift $#
        ;;
    editorAgentProfile)
        what="{\"kind\":\"editorAgentProfile\",\"agent\":\"$(json "$1")\"}"
        shift $#
        ;;
    forwarded)
        agent=$1
        [ $# -gt 0 ] && shift
        case $ATOLLO_LEVEL in
            manual | project) from=$ATOLLO_LEVEL ;;
            *) from=manual ;;
        esac
        what="{\"kind\":\"forwarded\",\"agent\":\"$(json "$agent")\",\"from\":\"$from\"}"
        ;;
    *)
        say "launch.sh: editorTask|editorAgentProfile|forwarded"
        exit 2
        ;;
esac

list=''
for arg in "$@"; do
    list="$list${list:+,}\"$(json "$arg")\""
done
# The folder as Windows names it under Git Bash, as it is elsewhere.
cwd=$(pwd -W 2>/dev/null) || cwd=$(pwd)

# The request, written whole then renamed: Atollo never reads half of one.
id=$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')
mkdir -p "$requests" || exit 1
request="$requests/$id.json"
printf '{"v":1,"origin":%s,"cwd":"%s","args":[%s]}' "$what" "$(json "$cwd")" "$list" >"$requests/$id.tmp" &&
    mv "$requests/$id.tmp" "$request" || exit 1

# The reply: within seconds, or minutes the first time a VM starts. Atollo
# takes the request at once while it runs: one still there after the wait
# was never taken, and is taken back.
reply="$requests/$id.reply.sh"
wait=${ATOLLO_LAUNCH_WAIT:-15}
case $wait in '' | *[!0-9]*) wait=15 ;; esac
start=$(date +%s)
said=''
while [ ! -f "$reply" ]; do
    waited=$(($(date +%s) - start))
    if [ "$waited" -ge "$wait" ] && [ -f "$request" ] && rm "$request" 2>/dev/null; then
        say "Atollo non è aperto: apri il terminale da Atollo."
        exit 1
    fi
    if [ "$waited" -ge 600 ]; then
        rm -f "$request"
        say "Atollo non ha risposto in 10 minuti: riprova, o guarda il registro di Atollo."
        exit 1
    fi
    if [ -z "$said" ] && [ "$waited" -ge 3 ]; then
        say "[Atollo] In preparazione..."
        said=1
    fi
    sleep 0.2 2>/dev/null || sleep 1
done

placement=''
program=''
marker=''
banner=''
text=''
. "$reply"
rm -f "$reply" "$requests/$id.reply.json"

case $placement in
    vm | pc) ;;
    *)
        say "$text"
        exit 1
        ;;
esac
[ -n "$banner" ] && say "$banner"
atollo_env

if [ -n "$marker" ]; then
    # Pi's session, seen by Atollo on the PC (D-178): a marker with this
    # shell's pid (Windows' own under Git Bash) while it runs, in a folder
    # no VM mounts.
    pid=$$
    [ -r "/proc/$$/winpid" ] && pid=$(cat "/proc/$$/winpid")
    mark="$requests/$marker.$pid"
    printf '%s\n' "$pid" >"$mark"
    trap 'rm -f "$mark"' EXIT
    trap ':' INT TERM HUP
    "$program" "$@"
    exit $?
fi
exec "$program" "$@"
