#!/bin/sh
# check-contrast.sh — text colour tokens must clear WCAG 2.1 AA against the real backgrounds.
#
# WHY THIS EXISTS
#   The C-2 explainer shipped with 31 of 122 text elements below 4.5:1. The cause was not
#   carelessness about colour — it was two tokens used for text that had never been
#   contrast-tested for text. `--ink-dim` measured 2.52:1. It was a decorative grey doing a
#   reading job on the page a grower opens outdoors, at sixty, on a phone.
#
#   The regression this guards is a token edited back toward "looks nicer". A lighter grey
#   always looks nicer on a designer's monitor indoors, and there is no feedback from the
#   person who cannot read it.
#
# WHAT IT CHECKS
#   Every declared text-colour token, against every declared background, at the AA threshold
#   for normal text (4.5:1). Deliberately strict: it does not grant the 3:1 large-text
#   allowance, because a token can be used at any size and the check cannot know where.
#
# WHAT IT DOES NOT CHECK, and will not pretend to
#   - per-element contrast as rendered. That needs a browser; it was done there, and this
#     guards the tokens those measurements depended on.
#   - alpha-composited backgrounds beyond the ones named below.
#   - whether text is large enough to read, which WCAG AA does not require and this project
#     cares about anyway.
#
# USAGE  sh verify/check-contrast.sh <surface> | selftest
# EXIT   0 pass · 1 a token fails AA · 2 REFUSED (no verdict formed)

set -u

# relative luminance + contrast ratio, per WCAG 2.1
ratio_of() {
    awk -v fg="$1" -v bg="$2" 'BEGIN{
        split(fg,f,","); split(bg,b,",")
        for(i=1;i<=3;i++){
            c=f[i]/255; f[i]=(c<=0.03928)?c/12.92:((c+0.055)/1.055)^2.4
            c=b[i]/255; b[i]=(c<=0.03928)?c/12.92:((c+0.055)/1.055)^2.4
        }
        L1=0.2126*f[1]+0.7152*f[2]+0.0722*f[3]
        L2=0.2126*b[1]+0.7152*b[2]+0.0722*b[3]
        hi=(L1>L2)?L1:L2; lo=(L1>L2)?L2:L1
        printf "%.2f", (hi+0.05)/(lo+0.05)
    }'
}

hex_to_rgb() { printf '%d,%d,%d' 0x${1#\#} 2>/dev/null >/dev/null;
    h=${1#\#}; printf '%d,%d,%d' $((0x${h%????})) $((0x$(echo $h|cut -c3-4))) $((0x$(echo $h|cut -c5-6))); }

run_check() {
    _f="$1"
    [ -f "$_f" ] || { echo "REFUSE: no such surface: $_f" >&2; return 2; }

    # text tokens whose job is to be read
    _tok="--ink --ink-mut --ink-dim --guard --info"
    # opaque backgrounds text sits on, plus the composited .gap surface measured in-browser
    _bgs="--paper --card"

    _n=0; _fail=0
    for t in $_tok; do
        _hex=$(grep -oE -e "$t:[[:space:]]*#[0-9A-Fa-f]{6}" "$_f" | head -1 | grep -oE '#[0-9A-Fa-f]{6}')
        [ -z "$_hex" ] && continue
        _fg=$(hex_to_rgb "$_hex")
        for b in $_bgs; do
            _bh=$(grep -oE -e "$b:[[:space:]]*#[0-9A-Fa-f]{6}" "$_f" | head -1 | grep -oE '#[0-9A-Fa-f]{6}')
            [ -z "$_bh" ] && continue
            _bg=$(hex_to_rgb "$_bh")
            _r=$(ratio_of "$_fg" "$_bg")
            _n=$((_n + 1))
            if awk -v r="$_r" 'BEGIN{exit !(r < 4.5)}'; then
                echo "  FAIL $t ($_hex) on $b ($_bh) = ${_r}:1  — AA needs 4.5"
                _fail=1
            fi
        done
        # the tinted callout background, composited: guard 7% over paper
        _gb=$(awk -v g="$(hex_to_rgb "$(grep -oE '\-\-guard:[[:space:]]*#[0-9A-Fa-f]{6}' "$_f" | head -1 | grep -oE '#[0-9A-Fa-f]{6}')")" \
                  -v p="$(hex_to_rgb "$(grep -oE '\-\-paper:[[:space:]]*#[0-9A-Fa-f]{6}' "$_f" | head -1 | grep -oE '#[0-9A-Fa-f]{6}')")" \
              'BEGIN{split(g,G,",");split(p,P,",");printf "%d,%d,%d",G[1]*0.07+P[1]*0.93,G[2]*0.07+P[2]*0.93,G[3]*0.07+P[3]*0.93}')
        _r=$(ratio_of "$_fg" "$_gb")
        _n=$((_n + 1))
        if awk -v r="$_r" 'BEGIN{exit !(r < 4.5)}'; then
            echo "  FAIL $t ($_hex) on the tinted callout = ${_r}:1  — AA needs 4.5"
            _fail=1
        fi
    done

    # LAW 1 — a verdict over an empty set is a missing test, not a pass.
    if [ "$_n" -eq 0 ]; then
        echo "REFUSE: measured 0 colour combinations in $_f" >&2
        echo "        Either the tokens are named differently or the extractor is broken." >&2
        return 2
    fi

    echo "combinations_checked=$_n"
    echo "RESULT=$([ "$_fail" -eq 0 ] && echo PASS || echo FAIL)"
    return "$_fail"
}

run_selftest() {
    _w="${TMPDIR:-/tmp}/contrast.$$"; rm -rf "$_w"; mkdir -p "$_w" || return 2
    _p=0; _f=0
    _mk() { printf ':root{ --paper:#F7F6F1; --card:#FFFFFF; --ink:#1E2320; --ink-mut:%s; --ink-dim:%s; --guard:%s; --info:#1F6CAB; }\n' "$2" "$3" "$4" > "$_w/$1.html"; }
    _case() { ( run_check "$_w/$1.html" ) >"$_w/$1.out" 2>&1; _g=$?
        if [ "$_g" -eq "$2" ]; then printf '  ok    %-26s exit=%s\n' "$1" "$_g"; _p=$((_p+1))
        else printf '  FAIL  %-26s exit=%s (expected %s)\n' "$1" "$_g" "$2"; sed 's/^/          /' "$_w/$1.out"; _f=$((_f+1)); fi }

    echo "selftest: the check must fail on the colours that actually shipped broken"

    _mk passing '#5A655D' '#5A655D' '#6E51A0';           _case passing 0

    # THE ONE THAT MATTERS — the exact --ink-dim that shipped at 2.52:1
    _mk shipped_ink_dim '#5A655D' '#93A096' '#6E51A0';   _case shipped_ink_dim 1

    # the guard violet that failed only on the tinted callout, at 4.03:1
    _mk shipped_guard '#5A655D' '#5A655D' '#7D5FB0';     _case shipped_guard 1

    # a "looks nicer" drift — one step lighter, still plausible, below threshold
    _mk drifted_lighter '#6B766F' '#6B766F' '#6E51A0';   _case drifted_lighter 1

    # no tokens at all must REFUSE, never pass
    printf '<p>no tokens here</p>\n' > "$_w/no_tokens.html";  _case no_tokens 2

    rm -rf "$_w"
    echo "selftest: $_p passed, $_f failed"
    [ "$_f" -eq 0 ] || return 1
    return 0
}

case "${1:-}" in
    selftest) run_selftest; exit $? ;;
    "")       echo "usage: $0 <surface> | selftest" >&2; exit 2 ;;
    *)        run_check "$1"; exit $? ;;
esac
