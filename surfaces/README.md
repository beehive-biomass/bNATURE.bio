# surfaces

Static, zero-external-reference surfaces.

## How these are delivered — the limitation, stated

- **The bytes fetch nothing.** No external reference of any kind.
- **Reading them is neither private nor guaranteed.** Served by one operator who can observe
  who reads them and withdraw them at will. Nothing here is censorship-resistant or
  unobserved. The same sentence appears in every artifact, so a reader arriving by direct
  link is told without having to find this file.

**No level is claimed.** No grant record exists for this tree yet.

## Clearance chain

| surface | sha256 |
|---|---|
| `c2_contract_explainer.html` | `bae544f14592a951324a271808da9f9fe160ca01842a3746c6887743e6626f52` <!-- PUBLIC-CONSTANT: content digest --> |

## C-2 · what this explainer is, and what checking it caught

Six explanations against the SinglePoint EX-10.1 fixture, in grower-exposure order. **One
explanation per contract, never per party** — the page is identical for whoever reads it,
because a tool that explains a document differently depending on who is asking is not
explaining the document. **Comprehension, not satisfaction:** each item names the standard a
term is measured by and none tells a reader what to do.

**Explanation 1 was rewritten twice before it was right, and the history matters.** The
original order carried the finding as *"285,000 lbs vs single occurrence, one month — the
contradiction no grower catches."* Reading the filed document disproved that twice. The
arithmetic reconciles perfectly (`45,000 + 240,000 = 285,000`); there is no contradiction.
What is actually there is that **84% of the commitment turns on *may* rather than *shall***,
three sections from the number it governs. The obligation does not contradict itself — it
becomes an option, in a paragraph about price.

Three passes, with full-text search, hunting deliberately for defects, and the first two
readings were both **more dramatic than the truth**. That is the argument for building from
the document rather than from anyone's account of it.

**Explanation 5 does not match the order either.** The order specified a
silence-equals-acceptance term. **The document contains no such clause.** It has a bounded
3-day inspection period and no statement of what happens when the window closes. The page
says that, rather than describing a term that is not there.

**Verification found two real defects in this page's own claims:**

1. The FOB quotation dropped `(Free On Board)` while the page claimed every quotation was
   verbatim. Restored.
2. An example of *typical* drafting sat in quotation marks on a page asserting all
   quotations came from this document. It is the term this contract **lacks** — quoting it
   was the wrong shape. Reworded.

Both were missed by a first check that tested **substrings chosen by the author**, several
truncated before apostrophes, which dodged exactly the places entities differ. A check that
avoids the hard parts is not a check. The second pass extracted every quotation whole.

**Redactions are disclosed first, before any explanation.** The filed document is redacted in
21 places including **every per-pound price**, so the page states plainly that it cannot say
what any of this pays.

**Not legal advice**, and no position is taken on whether these terms are unusual.

## The quotation rule, and why it is a script

**Verbatim quotations are in the operative language and are never translated.**

Quotation marks assert provenance. A quotation rendered in a language the document was not
written in asserts that those were the words — and they were not. So a Spanish explainer
quotes this contract **in English**, because English is what was signed and what an
arbitrator in Multnomah County reads. The explanation is translated; the quotation is not;
any plain-language gloss sits beside it, marked as a gloss rather than as the term.

`verify/check-quotations.sh` enforces it. Six fixtures, run before it shipped:

| fixture | expected |
|---|---|
| identical quotations | pass |
| **a quotation rendered in Spanish** | **fail** — plausible, well-meant, false provenance |
| a quotation silently dropped | fail |
| one word changed — `may` softened to `will` | fail |
| no operative-language declaration | fail |
| **no quotations at all** | **refuse, exit 2** |

**The check is a script rather than a style note deliberately.** This project has now fixed
the same class of defect three times by fixing the instance — a composition check written
and never run against its own levels; guards repaired in one tree while a sibling stayed
fail-open; `fixtures/** -text` added an hour before a digest-pinned file landed in
`surfaces/`. A rule nobody can run is a discipline, and disciplines decay.

**Writing it caught two defects in itself.** The extractor left ordinary prose intact in
files containing no blockquotes, so a page with zero quotations produced one phantom
quotation made of its own body text — worse than a miss, because a check that manufactures
quotations compares the wrong things everywhere. And a count used `grep -c`, which prints
`0` *and* exits non-zero, so the fallback appended a second `0` and the empty-set refusal
never fired. Both found by the selftest failing, not by reading the script.

**It also found a gap in this page:** the explainer quoted an English contract without ever
saying English is what governs. Now stated first, before any explanation.
