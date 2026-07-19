#!/bin/sh
# check-quotations.sh — a translated surface must not translate its quotations.
#
# THE RULE
#   Quotation marks assert provenance. A quotation rendered in a language the document was
#   not written in asserts that those were the words. They were not.
#
#   So: the explanation is translated; the quotation is not. Every quoted passage in a
#   translated surface must be BYTE-IDENTICAL to the same passage in the operative-language
#   surface, and the operative language must be declared in the artifact.
#
# WHY THIS IS A SCRIPT AND NOT A STYLE GUIDE
#   This project has now fixed the same class of defect three times by fixing the instance:
#   a composition check written and not run against its own levels; guards repaired in one
#   tree while a sibling stayed fail-open; `fixtures/** -text` added an hour before a
#   digest-pinned file landed in `surfaces/`. Each time the rule existed and the class did
#   not get covered. A rule nobody can run is a discipline, and disciplines decay.
#
# WHAT THIS DOES NOT CHECK, and will not pretend to
#   - whether the translation is any good. It cannot read Spanish.
#   - whether the gloss beside a quotation is accurate. That needs a speaker.
#   - whether the operative language declared is the true one. It checks that a declaration
#     exists, not that it is correct.
#
# USAGE
#   sh verify/check-quotations.sh <operative-surface> <translated-surface>
#   sh verify/check-quotations.sh selftest
#
# EXIT
#   0 every quotation matches · 1 a quotation was altered · 2 REFUSED (no verdict formed)

set -u

# Extract ONLY the contents of <blockquote> elements.
#
# An earlier version split on the closing tag and stripped the rest, which left ordinary
# prose untouched in files that contained no blockquotes at all — so a page with zero
# quotations yielded one phantom "quotation" made of its own body text. That is worse than a
# miss: a check that manufactures quotations out of prose compares the wrong things
# everywhere, and would have reported agreement between two pages that quote nothing.
#
# awk splits on the opening tag and keeps only what follows one, so a file with no
# blockquotes yields nothing — which is what lets the empty-set refusal below mean anything.
quotes_of() {
    tr '\n' ' ' < "$1" | awk '{
        n = split($0, parts, /<blockquote>/)
        for (i = 2; i <= n; i++) {
            split(parts[i], q, /<\/blockquote>/)
            print q[1]
        }
    }' \
      | sed -e 's|<cite>[^<]*</cite>||g' -e 's/<[^>]*>//g' \
      | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' \
      | tr -s ' ' \
      | grep -v '^$'
}

run_check() {
    _op="$1"; _tr="$2"
    [ -f "$_op" ] || { echo "REFUSE: operative surface not found: $_op" >&2; return 2; }
    [ -f "$_tr" ] || { echo "REFUSE: translated surface not found: $_tr" >&2; return 2; }

    quotes_of "$_op" > "${TMPDIR:-/tmp}/q-op.$$"
    quotes_of "$_tr" > "${TMPDIR:-/tmp}/q-tr.$$"
    _nop=$(awk "NF{n++} END{print n+0}" "${TMPDIR:-/tmp}/q-op.$$")
    _ntr=$(awk "NF{n++} END{print n+0}" "${TMPDIR:-/tmp}/q-tr.$$")

    # LAW 1 — non-emptiness before any verdict. Two empty quote sets compare equal, and
    # "identical" over nothing is the vacuous pass this project keeps catching.
    if [ "$_nop" -eq 0 ]; then
        echo "REFUSE: extracted 0 quotations from the operative surface." >&2
        echo "        A comparison over an empty set is a missing test, not a pass." >&2
        rm -f "${TMPDIR:-/tmp}/q-op.$$" "${TMPDIR:-/tmp}/q-tr.$$"; return 2
    fi
    if [ "$_ntr" -eq 0 ]; then
        echo "REFUSE: extracted 0 quotations from the translated surface." >&2
        rm -f "${TMPDIR:-/tmp}/q-op.$$" "${TMPDIR:-/tmp}/q-tr.$$"; return 2
    fi

    echo "quotations_operative=$_nop quotations_translated=$_ntr"

    _fail=0
    if [ "$_nop" -ne "$_ntr" ]; then
        echo "  COUNT MISMATCH: a translated surface must quote the same passages"
        _fail=1
    fi

    # every quotation in the translated surface must appear byte-identical in the operative one
    _i=0
    while IFS= read -r q; do
        _i=$((_i + 1))
        if ! grep -qxF "$q" "${TMPDIR:-/tmp}/q-op.$$"; then
            echo "  ALTERED QUOTATION #$_i — not byte-identical to the operative surface:"
            echo "    ${q}" | cut -c1-100
            _fail=1
        fi
    done < "${TMPDIR:-/tmp}/q-tr.$$"

    # the artifact must declare which language its quotations are in
    if grep -qiE 'operative language|idioma operativo|language of the (contract|agreement|document)' "$_tr"; then
        echo "operative_language_declared=yes"
    else
        echo "  NO OPERATIVE-LANGUAGE DECLARATION in $_tr"
        echo "     A reader cannot tell whose words are quoted unless the artifact says."
        _fail=1
    fi

    rm -f "${TMPDIR:-/tmp}/q-op.$$" "${TMPDIR:-/tmp}/q-tr.$$"
    echo "RESULT=$([ "$_fail" -eq 0 ] && echo PASS || echo FAIL)"
    return "$_fail"
}

# ── selftest · Law 2. The check does not ship until it has been watched biting. ──
run_selftest() {
    _w="${TMPDIR:-/tmp}/quotecheck.$$"; rm -rf "$_w"; mkdir -p "$_w" || return 2
    _pass=0; _failed=0

    _mkpair() { # dir
        mkdir -p "$_w/$1"
        cat > "$_w/$1/op.html" <<'HTML'
<p>Explanation in the operative language.</p>
<blockquote>"Supplier will sell and Buyer will purchase 285,000 lbs"<cite>Purchase Commitment</cite></blockquote>
<blockquote>"Buyer may place monthly Purchase Orders in excess of the Initial Order."<cite>Pricing</cite></blockquote>
HTML
        cat > "$_w/$1/tr.html" <<'HTML'
<p>Explicacion en espanol. El idioma operativo del contrato es el ingles.</p>
<blockquote>"Supplier will sell and Buyer will purchase 285,000 lbs"<cite>Purchase Commitment</cite></blockquote>
<blockquote>"Buyer may place monthly Purchase Orders in excess of the Initial Order."<cite>Pricing</cite></blockquote>
HTML
    }

    _case() {
        ( run_check "$_w/$1/op.html" "$_w/$1/tr.html" ) >"$_w/$1.out" 2>&1
        _got=$?
        if [ "$_got" -eq "$2" ]; then
            printf '  ok    %-26s exit=%s (expected %s)\n' "$1" "$_got" "$2"; _pass=$((_pass+1))
        else
            printf '  FAIL  %-26s exit=%s (expected %s)\n' "$1" "$_got" "$2"
            sed 's/^/          /' "$_w/$1.out"; _failed=$((_failed+1))
        fi
    }

    echo "selftest: the check must fail when a quotation is translated"

    _mkpair intact;                                              _case intact 0

    # THE ONE THAT MATTERS: the quotation rendered in Spanish. Plausible, well-meant,
    # and a false claim of provenance — those were not the words in the document.
    _mkpair translated_quote
    sed -i.bak 's|"Buyer may place monthly Purchase Orders in excess of the Initial Order."|"El Comprador podra emitir Ordenes de Compra mensuales."|' \
        "$_w/translated_quote/tr.html" 2>/dev/null
    rm -f "$_w/translated_quote/"*.bak
    _case translated_quote 1

    # a quotation silently dropped from the translated surface
    _mkpair dropped_quote
    sed -i.bak '/Initial Order/d' "$_w/dropped_quote/tr.html" 2>/dev/null
    rm -f "$_w/dropped_quote/"*.bak
    _case dropped_quote 1

    # a single word changed inside a quotation — "may" softened to "will"
    _mkpair reworded_quote
    sed -i.bak 's|Buyer may place monthly|Buyer will place monthly|' "$_w/reworded_quote/tr.html" 2>/dev/null
    rm -f "$_w/reworded_quote/"*.bak
    _case reworded_quote 1

    # no operative-language declaration
    _mkpair no_declaration
    sed -i.bak 's|El idioma operativo del contrato es el ingles.||' "$_w/no_declaration/tr.html" 2>/dev/null
    rm -f "$_w/no_declaration/"*.bak
    _case no_declaration 1

    # empty quote set must REFUSE, never pass
    _mkpair empty_set
    printf '<p>no quotations here</p>\n' > "$_w/empty_set/tr.html"
    _case empty_set 2

    rm -rf "$_w"
    echo "selftest: $_pass passed, $_failed failed"
    [ "$_failed" -eq 0 ] || return 1
    return 0
}

case "${1:-}" in
    selftest) run_selftest; exit $? ;;
    "")       echo "usage: $0 <operative-surface> <translated-surface> | selftest" >&2; exit 2 ;;
    *)        [ $# -eq 2 ] || { echo "usage: $0 <operative> <translated> | selftest" >&2; exit 2; }
              run_check "$1" "$2"; exit $? ;;
esac
