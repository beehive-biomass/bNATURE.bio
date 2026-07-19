#!/bin/sh
# check-figures.sh — a numeral written twice must agree with itself.
#
# THE RULE
#   Figures a reader is asked to act on or compare are written twice: the numeral exactly as
#   the document has it, then the same figure in words. That is the contract's own
#   convention — the filed text reads "285,000 (two hundred eighty five thousand) lbs" — and
#   it is the oldest anti-ambiguity device in drafting.
#
#   It matters here because numerals are read differently in different places. `285,000` is
#   `285.000` in es-ES, and `285.000` read by an English speaker is "two hundred eighty-five
#   point zero zero zero". Words do not have that problem. So when this page is published in
#   another language the numeral stays frozen exactly as filed and the WORDS are translated.
#
#   Which creates the failure this script exists for: **someone edits one and not the other.**
#   A numeral and its words that disagree is worse than either alone, because the page now
#   states two different quantities with equal confidence.
#
# WHAT THIS DOES NOT CHECK
#   - whether the figure matches the source document. That is a separate concern.
#   - Spanish or any other language's words. It converts English only, and refuses rather
#     than guessing when it meets words it cannot generate.
#
# USAGE   sh verify/check-figures.sh <surface> | selftest
# EXIT    0 agree · 1 disagree · 2 REFUSED (no verdict formed)

set -u

# numeral -> English words, for the magnitudes this domain actually uses (< 1,000,000).
# Refuses outside that range rather than emitting something plausible and wrong.
words_for() {
    _n=$(printf '%s' "$1" | tr -d ',')
    case "$_n" in *[!0-9]*|'') return 2 ;; esac
    [ "$_n" -ge 1000000 ] && return 2
    _u="one two three four five six seven eight nine"
    _teen="ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen"
    _t="twenty thirty forty fifty sixty seventy eighty ninety"
    _under1000() {
        _v=$1; _o=""
        [ "$_v" -ge 100 ] && { _o="$(echo $_u | cut -d' ' -f$((_v/100))) hundred"; _v=$((_v%100)); }
        if [ "$_v" -ge 20 ]; then
            _o="$_o $(echo $_t | cut -d' ' -f$((_v/10-1)))"; _v=$((_v%10))
            [ "$_v" -gt 0 ] && _o="$_o $(echo $_u | cut -d' ' -f$_v)"
        elif [ "$_v" -ge 10 ]; then
            _o="$_o $(echo $_teen | cut -d' ' -f$((_v-9)))"
        elif [ "$_v" -gt 0 ]; then
            _o="$_o $(echo $_u | cut -d' ' -f$_v)"
        fi
        printf '%s' "$_o"
    }
    _out=""
    _k=$((_n/1000)); _r=$((_n%1000))
    [ "$_k" -gt 0 ] && _out="$(_under1000 $_k) thousand"
    [ "$_r" -gt 0 ] && _out="$_out $(_under1000 $_r)"
    printf '%s' "$_out" | sed 's/^ *//; s/  */ /g'
}

# Hyphenation is orthographic, not numeric. The filed contract writes "eighty five" and
# ordinary English writes "eighty-five"; both name the same number. Normalising hyphens is
# therefore honest — normalising anything that changes the VALUE would not be.
norm_words() {
    tr 'A-Z' 'a-z' | tr '-' ' ' | sed 's/ and / /g; s/^ *//; s/ *$//; s/  */ /g'
}

run_check() {
    _f="$1"
    [ -f "$_f" ] || { echo "REFUSE: no such surface: $_f" >&2; return 2; }

    # pairs of the form  123,456 ... (words)
    sed 's/<[^>]*>/ /g' "$_f" | tr -s ' ' \
      | grep -oE '[0-9]{1,3},[0-9]{3} *\([a-zA-Z -]+\)' > "${TMPDIR:-/tmp}/pairs.$$" || true
    _n=$(awk 'NF{c++} END{print c+0}' "${TMPDIR:-/tmp}/pairs.$$")

    # LAW 1 — a verdict over an empty set is a missing test, not a pass.
    if [ "$_n" -eq 0 ]; then
        echo "REFUSE: found 0 numeral-and-words pairs in $_f" >&2
        echo "        Either the convention was not applied, or the extractor is broken." >&2
        echo "        Both are reasons to refuse rather than report agreement." >&2
        rm -f "${TMPDIR:-/tmp}/pairs.$$"; return 2
    fi
    echo "figure_pairs=$_n"

    _fail=0
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        _num=$(printf '%s' "$line" | grep -oE '^[0-9,]+')
        _wds=$(printf '%s' "$line" | sed 's/^[0-9,]* *(//; s/)$//')
        if ! _gen=$(words_for "$_num"); then
            echo "  REFUSE: cannot generate words for $_num (outside supported range)"
            _fail=2; continue
        fi
        _a=$(printf '%s' "$_gen" | norm_words)
        _b=$(printf '%s' "$_wds" | norm_words)
        if [ "$_a" = "$_b" ]; then
            printf '  ok   %-9s (%s)\n' "$_num" "$_wds"
        else
            echo "  DISAGREE $_num"
            echo "    page says : $_wds"
            echo "    numeral is: $_gen"
            _fail=1
        fi
    done < "${TMPDIR:-/tmp}/pairs.$$"
    rm -f "${TMPDIR:-/tmp}/pairs.$$"

    [ "$_fail" -eq 2 ] && { echo "RESULT=REFUSED"; return 2; }
    echo "RESULT=$([ "$_fail" -eq 0 ] && echo PASS || echo FAIL)"
    return "$_fail"
}

run_selftest() {
    _w="${TMPDIR:-/tmp}/figcheck.$$"; rm -rf "$_w"; mkdir -p "$_w" || return 2
    _p=0; _f=0
    _mk() { printf '<p>firm subtotal 45,000 (forty-five thousand) lb and 240,000 (two hundred forty thousand) lb</p>\n' > "$_w/$1.html"; }
    _case() {
        ( run_check "$_w/$1.html" ) > "$_w/$1.out" 2>&1; _g=$?
        if [ "$_g" -eq "$2" ]; then printf '  ok    %-24s exit=%s\n' "$1" "$_g"; _p=$((_p+1))
        else printf '  FAIL  %-24s exit=%s (expected %s)\n' "$1" "$_g" "$2"; sed 's/^/          /' "$_w/$1.out"; _f=$((_f+1)); fi
    }
    echo "selftest: numeral and words must be checked against each other, not assumed"

    _mk agree; _case agree 0

    # the failure this exists for: the numeral edited, the words left behind
    _mk numeral_edited
    sed -i.bak 's/45,000 (forty-five/54,000 (forty-five/' "$_w/numeral_edited.html" 2>/dev/null; rm -f "$_w"/*.bak
    _case numeral_edited 1

    # and the mirror: words edited, numeral left behind
    _mk words_edited
    sed -i.bak 's/(two hundred forty thousand)/(two hundred fourteen thousand)/' "$_w/words_edited.html" 2>/dev/null; rm -f "$_w"/*.bak
    _case words_edited 1

    # the contract's own unhyphenated style must still agree — hyphenation is orthographic
    printf '<p>285,000 (two hundred eighty five thousand) lbs</p>\n' > "$_w/unhyphenated.html"
    _case unhyphenated 0

    # no pairs at all must REFUSE, never pass
    printf '<p>no figures written twice here</p>\n' > "$_w/no_pairs.html"
    _case no_pairs 2

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
