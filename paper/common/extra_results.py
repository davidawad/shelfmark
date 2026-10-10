#!/usr/bin/env python3
"""Values and tables shared by paper/paper1 and paper/method-survey.

Everything here is read from committed JSON under the repository root; no
number is typed by hand. The two table scripts import `collect()` and render
its output as Markdown (paper1) or Typst (method-survey).

`collect(root)` returns `(values, tables)`:

* `values`: dict name -> display string (prefix `x_`);
* `tables`: dict name -> `{"header": [...], "rows": [[...], ...]}`.

Sources (all committed):

* docs/validity/results.json            judge agreement for Dasgupta cost
* docs/validity/falsejoin-results.json  the same labels, false-join variants
* bench/results/2026-10-11-stability/   paper-corpus rank stability
* bench/results/2026-10-11-book-stability/   book claims over 27 settings
* bench/results/2026-10-10-covered/     coverage analysis
* bench/results/2026-10-10-embed/       cross-fit of the learned scheme
* bench/results/2026-10-10-semantic/    lexical vs shortlist tier
* bench/results/2026-10-10-learned/     learned scheme

Standard library only.
"""
import collections
import json
import statistics
from pathlib import Path

OBJ_LABEL = {
    "dasgupta_cost_norm": "Dasgupta cost",
    "moseley_wang_revenue": "Moseley-Wang revenue",
    "skill": "skill",
    "triplet_accuracy": "triplet accuracy",
    "shelf_order_locality": "shelf-order locality",
}

# Order of the 15 entries in docs/validity/falsejoin-results.json
# groups[*]: Dasgupta, the two false-join terms alone, then Dasgupta plus
# beta times each term for beta in BETAS.
BETAS = ["0.25", "0.5", "1", "2", "4", "8"]
VARIANTS = (["dasgupta", "fj", "fjs"]
            + [f"c+{b}fj" for b in BETAS] + [f"c+{b}fjs" for b in BETAS])

RUN_ORDER = ["arxiv-gold", "msc-gold", "fdc-placed", "ilc2-placed",
             "lcc-full-placed", "openalex-topics-placed",
             "unesco-thesaurus-placed", "learned-placed"]

CLAIM_ROWS = [
    # (claim id, objective key, label of the claim as written in report 9)
    ("dewey-vs-lcc", "C", "Dewey sections vs LCC, librarian numbers"),
    ("dewey-vs-lcc", "J'", "Dewey sections vs LCC, librarian numbers"),
    ("section-vs-fdc-d3", "C", "Dewey sections vs FDC depth 3"),
    ("cut-3-vs-2", "J'", "FDC depth 3 vs depth 2"),
    ("cut-3-vs-full", "J'", "FDC depth 3 vs full FDC number"),
    ("full-vs-section", "J'", "Full Dewey number vs section"),
    ("learned-vs-section", "J'", "Learned scheme vs Dewey sections"),
    ("fdc-placed-vs-lcc-placed", "J'", "Placed: FDC vs LCC outline"),
    ("fdc-placed-vs-lcc-lcsh-placed", "J'", "Placed: FDC vs LCC outline + LCSH"),
]

SIGN = {-1: "a < b", 1: "a > b", 0: "tie"}


def _load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _f2(x):
    return f"{x:.2f}"


def _f3(x):
    return f"{x:.3f}"


def _s3(x):
    return f"{round(x, 3) + 0.0:+.3f}"


def _s4(x):
    return f"{round(x, 4) + 0.0:+.4f}"


def _ci2(lo, hi):
    return f"[{lo:.2f}, {hi:.2f}]"


def _ranks(v):
    order = sorted(range(len(v)), key=lambda i: v[i])
    r = [0.0] * len(v)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and v[order[j + 1]] == v[order[i]]:
            j += 1
        for k in range(i, j + 1):
            r[order[k]] = (i + j) / 2 + 1
        i = j + 1
    return r


def spearman(x, y):
    rx, ry = _ranks(x), _ranks(y)
    mx, my = sum(rx) / len(rx), sum(ry) / len(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = (sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry)) ** 0.5
    return num / den


def _validity(root, V, T):
    res = _load(root / "docs/validity/results.json")
    fj = _load(root / "docs/validity/falsejoin-results.json")
    a = res["all"]
    V["x_val_n"] = f"{a['n']:,}"
    V["x_val_kappa"] = _f2(a["kappa"])
    lo, hi = a["ci"]["kappa"]
    V["x_val_kappa_lo"], V["x_val_kappa_hi"] = _f2(lo), _f2(hi)
    V["x_val_agree"] = _f2(a["agree"])
    V["x_val_auc"] = _f2(a["auc"])
    V["x_val_judge_incorrect"] = _f2(a["judge_incorrect"])
    V["x_val_flagged"] = _f2(a["metric_flagged"])
    V["x_val_together_kappa"] = _f2(a["together_kappa"])
    V["x_val_scheme_together_kappa"] = f"{a['scheme_vs_judge_together_kappa']:.2f}"
    V["x_val_self_kappa"] = _f2(res["consistency"]["scheme_decision"]["kappa"])
    V["x_val_rho_c"] = _f2(res["cross_run"]["spearman_cost_vs_C_incorrect"])
    V["x_val_rho_all"] = _f2(res["cross_run"]["spearman_cost_vs_all_incorrect"])
    V["x_val_runs"] = str(len(res["runs"]))

    pooled = fj["groups"]["all runs"]
    assert len(pooled) == len(VARIANTS)
    ix = {v: i for i, v in enumerate(VARIANTS)}
    c1 = pooled[ix["c+1fjs"]]
    V["x_val_kappa_fj"] = _f2(c1["kappa"])
    V["x_val_kappa_fj_lo"], V["x_val_kappa_fj_hi"] = _f2(c1["kappa_ci"][0]), _f2(c1["kappa_ci"][1])
    V["x_val_dkappa_fj"] = f"{c1['dkappa']:+.2f}"
    V["x_val_dkappa_fj_lo"] = f"{c1['dkappa_ci'][0]:+.2f}"
    V["x_val_dkappa_fj_hi"] = f"{c1['dkappa_ci'][1]:+.2f}"
    V["x_val_auc_fj"] = _f2(c1["auc"])
    V["x_val_auc_base"] = _f2(pooled[0]["auc"])
    fjs_alone = pooled[ix["fjs"]]
    V["x_val_kappa_fjs_alone"] = _f2(fjs_alone["kappa"])
    c4 = pooled[ix["c+4fjs"]]
    V["x_val_kappa_fj_b4"] = _f2(c4["kappa"])

    rows = []
    for name in ["dasgupta", "fjs", "c+0.5fjs", "c+1fjs", "c+4fjs", "c+1fj"]:
        g = pooled[ix[name]]
        label = {"dasgupta": "Dasgupta cost C", "fjs": "FJs alone",
                 "c+0.5fjs": "C + 0.5 FJs", "c+1fjs": "C + FJs",
                 "c+4fjs": "C + 4 FJs", "c+1fj": "C + FJ (outside mass)"}[name]
        rows.append([label, _f2(g["flagged"]),
                     f"{_f2(g['kappa'])} {_ci2(*g['kappa_ci'])}",
                     f"{_f2(g['auc'])} {_ci2(*g['auc_ci'])}"])
    T["validity_variants"] = {
        "header": ["variant", "flagged", "kappa [95% CI]", "AUC [95% CI]"],
        "rows": rows}

    rows = []
    for run in ["arxiv-gold", "msc-gold", "fdc-placed", "ilc2-placed",
                "lcc-full-placed", "openalex-topics-placed",
                "unesco-thesaurus-placed", "learned-placed"]:
        g0, g1 = fj["groups"][run][ix["dasgupta"]], fj["groups"][run][ix["c+1fjs"]]
        rows.append([run, f"{_f2(g0['kappa'])} {_ci2(*g0['kappa_ci'])}",
                     f"{_f2(g1['kappa'])} {_ci2(*g1['kappa_ci'])}",
                     f"{g1['dkappa']:+.2f} [{g1['dkappa_ci'][0]:+.2f}, {g1['dkappa_ci'][1]:+.2f}]"])
    T["validity_runs"] = {
        "header": ["run", "kappa, C [95% CI]", "kappa, C + FJs [95% CI]", "change in kappa"],
        "rows": rows}


def _stability(root, V, T):
    st = _load(root / "bench/results/2026-10-11-stability/results.json")
    rm = st["rank_matrix"]
    n_settings = len(next(iter(rm.values()))["scores"]) and len(
        {k.split("|", 1)[1] for k in rm})
    V["x_stab_settings"] = str(n_settings)
    V["x_stab_threshold"] = f"{st['threshold_for_claims']:.1f}"
    by = collections.defaultdict(list)
    for p in st["pairwise_setting_agreement"]:
        same = p["setting_a"].split("|")[-1] == p["setting_b"].split("|")[-1]
        by[(p["objective"], same)].append(p["spearman_rho"])
    wins = collections.defaultdict(lambda: [0, 0])
    for r in rm.values():
        o = r["objective"]
        wins[o][1] += 1
        if r["ranks"]["fdc"] < r["ranks"]["lcc"]:
            wins[o][0] += 1
    rows = []
    for o in OBJ_LABEL:
        s, d = by[(o, True)], by[(o, False)]
        rows.append([OBJ_LABEL[o],
                     _f3(min(s)), _f3(statistics.median(s)),
                     _f3(min(d)), _f3(statistics.median(d)),
                     f"{wins[o][0]} of {wins[o][1]}"])
        key = {"dasgupta_cost_norm": "cost", "triplet_accuracy": "triplet",
               "shelf_order_locality": "locality", "skill": "skill",
               "moseley_wang_revenue": "revenue"}[o]
        V[f"x_stab_{key}_min_same"] = _f2(min(s))
        V[f"x_stab_{key}_min_diff"] = _f2(min(d))
        V[f"x_stab_{key}_med_diff"] = _f2(statistics.median(d))
        V[f"x_stab_{key}_fdc_wins"] = str(wins[o][0])
    V["x_stab_pairs_same"] = f"{len(by[('dasgupta_cost_norm', True)]):,}"
    V["x_stab_pairs_diff"] = f"{len(by[('dasgupta_cost_norm', False)]):,}"
    leaders = collections.defaultdict(collections.Counter)
    for key, r in rm.items():
        tier = key.split("|")[-1]
        leaders[(r["objective"], tier)][r["ranking"][0]] += 1
    cost_lead = {t: leaders[("dasgupta_cost_norm", t)].most_common(1)[0][0]
                 for t in ("lexical", "shortlist")}
    V["x_stab_lead_lexical"] = cost_lead["lexical"]
    V["x_stab_lead_shortlist"] = cost_lead["shortlist"]
    T["stability"] = {
        "header": ["objective", "min rho, same tier", "median rho, same tier",
                   "min rho, across tiers", "median rho, across tiers",
                   "FDC above LCC (settings)"],
        "rows": rows}


def _books(root, V, T):
    bs = _load(root / "bench/results/2026-10-11-book-stability/results.json")
    claims = {c["id"]: c for c in bs["claims"]}
    rows = []
    for cid, obj, label in CLAIM_ROWS:
        c = claims[cid]
        o = c["objectives"][obj]
        lo, hi = o["diff_range"]
        n = o["settings"]
        rows.append([label, "J'" if obj == "J'" else "C", SIGN[o["claimed_sign"]],
                     f"{o['share_sign_matches'] * n:.0f} of {n}",
                     f"{o['significant_for_claim'] if o['significant_for_claim'] is not None else '-'}"
                     f" / {o['significant_against_claim']}",
                     f"[{lo:+.4f}, {hi:+.4f}]", o["verdict"].lower()])
    T["book_claims"] = {
        "header": ["comparison (a vs b)", "objective", "claimed", "sign matches",
                   "significant for / against", "range of a - b", "verdict"],
        "rows": rows}

    d = claims["dewey-vs-lcc"]["objectives"]
    for obj, key in (("C", "c"), ("J'", "jp")):
        bset = d[obj]["by_setting"]
        for emb in ("minilm", "bge-small", "nomic"):
            s = bset[f"{emb}-k20-q00"]
            tag = emb.replace("-", "")
            V[f"x_dl_{key}_{tag}"] = _s4(s["diff"])
            V[f"x_dl_{key}_{tag}_lo"] = _s4(s["ci"][0])
            V[f"x_dl_{key}_{tag}_hi"] = _s4(s["ci"][1])
        V[f"x_dl_{key}_match"] = str(round(d[obj]["share_sign_matches"] * d[obj]["settings"]))
        V[f"x_dl_{key}_for"] = str(d[obj]["significant_for_claim"] or 0)
        V[f"x_dl_{key}_against"] = str(d[obj]["significant_against_claim"])
        V[f"x_dl_{key}_min"] = _s4(d[obj]["diff_range"][0])
        V[f"x_dl_{key}_max"] = _s4(d[obj]["diff_range"][1])
    V["x_dl_settings"] = str(d["C"]["settings"])
    reg = bs["regret"]
    V["x_regret_ddc9"] = _f3(reg["published_graphs_only"]["ddc-section"])
    V["x_regret_lcc9"] = _f3(reg["published_graphs_only"]["lcc-gold"])
    V["x_regret_ddc27"] = _f3(reg["all_settings"]["ddc-section"])
    V["x_regret_lcc27"] = _f3(reg["all_settings"]["lcc-gold"])
    rk = claims["rank-structures"]["objectives"]
    V["x_rank_struct_min_c"] = f"{rk['C']['min_spearman']:.2f}"
    V["x_rank_struct_min_j"] = "{:.2f}".format(rk["J'"]["min_spearman"])

    # coverage floor and placed-run winners
    br = bs["bead_rule"]["books"]
    V["x_bead_floor"] = f"{bs['coverage_floor']:.2f}"
    cov = br["coverage"]
    V["x_bead_cov_fdc"] = _f2(cov["fdc-placed"])
    V["x_bead_cov_lcclcsh"] = _f2(cov["lcc-lcsh-placed"])
    V["x_bead_cov_lcc"] = _f2(cov["lcc-placed"])
    hold = bs["bead_rule"]["holdout"]["coverage"]
    V["x_hold_best_cov"] = _f2(max(hold.values()))

    # learned scheme: margin over Dewey sections on each embedding's graph
    pts = bs["points"]["books"]
    rows = []
    for emb in ("minilm", "bge-small", "nomic"):
        s = pts[f"{emb}-k20-q00"]
        dc = s["learned-own"]["C"] - s["ddc-section"]["C"]
        dj = s["learned-own"]["J'"] - s["ddc-section"]["J'"]
        rows.append([emb + (" (fit graph)" if emb == "minilm" else ""),
                     _f3(s["learned-own"]["C"]), _f3(s["ddc-section"]["C"]),
                     _s3(dc), _s3(dj)])
        tag = emb.replace("-", "")
        V[f"x_learn_dc_{tag}"] = _s3(dc)
        V[f"x_learn_dj_{tag}"] = _s3(dj)
    T["learned_margin"] = {
        "header": ["graph (books, k = 20)", "C learned", "C Dewey sections",
                   "C difference", "J' difference"],
        "rows": rows}

    # cross-fit of the learned scheme (held-out split skill)
    emb = _load(root / "bench/results/2026-10-10-embed/results.json")
    cf = emb["cross_fit"]
    names = ["minilm", "bge-small", "nomic"]
    rows = []
    for fit in names:
        rows.append([f"fitted on {fit}"] +
                    [_f3(cf[f"{fit}@{sc}-k20"]["split_skill"]) for sc in names])
    T["cross_fit"] = {"header": ["learned fit"] + [f"scored on {n}" for n in names],
                      "rows": rows}
    diag = [cf[f"{n}@{n}-k20"]["split_skill"] for n in names]
    off = [cf[f"{f}@{s}-k20"]["split_skill"] for f in names for s in names if f != s]
    V["x_cf_diag_mean"] = _f3(sum(diag) / len(diag))
    V["x_cf_off_mean"] = _f3(sum(off) / len(off))
    V["x_cf_diag_wins"] = str(sum(
        1 for s in names
        if cf[f"{s}@{s}-k20"]["split_skill"] == max(cf[f"{f}@{s}-k20"]["split_skill"] for f in names)))


def _covered(root, V, T):
    cv = _load(root / "bench/results/2026-10-10-covered/results.json")
    g = cv["analysis"]["groups"]["placed-corpus"]["agree"]
    i = g["metrics"].index("dasgupta.cost_norm")
    c = g["control"]
    V["x_cov_rho"] = _f2(c["rho"][i])
    V["x_cov_rho_sq"] = f"{100 * c['rho_sq'][i]:.0f}%"
    V["x_cov_runs"] = str(g["compare"]["dasgupta.cost_norm"]["runs"])
    V["x_cov_tau"] = _f2(g["compare"]["dasgupta.cost_norm"]["kendall"])
    V["x_cov_msc_both"] = f"{cv['both']['msc-gold']['items']:,}"
    V["x_cov_msc_items"] = f"{cv['families']['msc-gold']['items']:,}"
    d = cv["dense"]["corpus"]
    sp = []
    for size, lst in d["sizes"].items():
        for e in lst:
            sp.append(e["agreement"]["graph-restricted"]["placed"]["spearman"])
    V["x_dense_min_rho"] = _f2(min(sp))


def _semantic(root, V, T):
    sdir = root / "bench/results/2026-10-10-semantic"
    schemes = ["acm-ccs", "arxiv", "bk", "eurovoc", "fdc", "ilc2", "ipc", "jel",
               "lcc", "lcc-full", "msc", "openalex-topics", "unesco-thesaurus"]
    rows, lx, sh = [], [], []
    for s in schemes:
        a = _load(sdir / f"{s}-placed.lexical.json")
        b = _load(sdir / f"{s}-placed.shortlist.json")
        lx.append(a["skill"])
        sh.append(b["skill"])
        rows.append([s, f"{a['coverage']:.3f}", f"{b['coverage']:.3f}",
                     _f3(a["skill"]), _f3(b["skill"]), _s3(b["skill"] - a["skill"])])
    T["semantic_tier"] = {
        "header": ["scheme", "coverage lexical", "coverage shortlist",
                   "skill lexical", "skill shortlist", "change in skill"],
        "rows": rows}
    V["x_sem_rho"] = _f2(spearman(lx, sh))
    order = sorted(range(len(schemes)), key=lambda i: -sh[i])
    V["x_sem_best"] = schemes[order[0]]
    V["x_sem_best_skill"] = _f3(sh[order[0]])
    V["x_sem_second"] = schemes[order[1]]
    V["x_sem_second_skill"] = _f3(sh[order[1]])
    V["x_sem_arxiv_lex"] = _f3(lx[schemes.index("arxiv")])
    V["x_sem_arxiv_sh"] = _f3(sh[schemes.index("arxiv")])
    V["x_sem_up"] = str(sum(1 for a, b in zip(lx, sh) if b > a))
    V["x_sem_n"] = str(len(schemes))
    V["x_sem_min_cov"] = f"{min(_load(sdir / f'{s}-placed.shortlist.json')['coverage'] for s in schemes):.3f}"
    V["x_sem_msc_lex"] = _f3(lx[schemes.index("msc")])
    V["x_sem_msc_sh"] = _f3(sh[schemes.index("msc")])
    V["x_sem_lccfull_lex"] = _f3(lx[schemes.index("lcc-full")])
    V["x_sem_lccfull_sh"] = _f3(sh[schemes.index("lcc-full")])
    V["x_sem_fdc_lex"] = _f3(lx[schemes.index("fdc")])
    V["x_sem_fdc_sh"] = _f3(sh[schemes.index("fdc")])
    V["x_sem_lcc_lex"] = _f3(lx[schemes.index("lcc")])
    V["x_sem_lcc_sh"] = _f3(sh[schemes.index("lcc")])
    # tiers whose placement files are identical
    res = _load(sdir / "results.json")
    V["x_sem_features"] = ", ".join(res["classifier"]["features"])


def _percentile(sorted_vals, q):
    # linear interpolation between order statistics
    pos = q * (len(sorted_vals) - 1)
    lo = int(pos)
    hi = min(lo + 1, len(sorted_vals) - 1)
    return sorted_vals[lo] + (sorted_vals[hi] - sorted_vals[lo]) * (pos - lo)


def _external(root, V, T):
    d = _load(root / "bench/results/2026-10-11-falsejoin/external/agreement.json")
    pairs = {"lcsh": ("ddc-section", "lcc-gold"),
             "coupling": ("fdc-arxiv-nested", "lcc-arxiv-nested"),
             "cocitation": ("fdc-arxiv-nested", "lcc-arxiv-nested")}
    names = {"lcsh": "shared LCSH heading (books)",
             "coupling": "bibliographic coupling (papers)",
             "cocitation": "co-citation (papers)"}
    rows = []
    for sig in ("lcsh", "coupling", "cocitation"):
        s = d["signals"][sig]
        m = s["models"]["minilm"]["variants"]
        a, b = pairs[sig]
        diffs = sorted(x - y for x, y in zip(s["separation_draws"][a], s["separation_draws"][b]))
        point = s["separation"][a]["point"] - s["separation"][b]["point"]
        lo, hi = _percentile(diffs, 0.025), _percentile(diffs, 0.975)
        k0, k1 = m["dasgupta"]["kappa"], m["c+1fjs"]["kappa"]
        r0, r1 = m["dasgupta"]["rho"], m["c+1fjs"]["rho"]
        rows.append([names[sig], _f2(k0), _f2(k1), _f2(r0), _f2(r1),
                     f"{a} - {b}: {point:+.3f} [{lo:+.3f}, {hi:+.3f}]"])
        V[f"x_ext_{sig}_k0"], V[f"x_ext_{sig}_k1"] = _f2(k0), _f2(k1)
        V[f"x_ext_{sig}_diff"] = f"{point:+.3f}"
        V[f"x_ext_{sig}_lo"], V[f"x_ext_{sig}_hi"] = f"{lo:+.3f}", f"{hi:+.3f}"
    T["external"] = {
        "header": ["external signal", "kappa, C", "kappa, C + FJs",
                   "scheme-level rho, C", "scheme-level rho, C + FJs",
                   "FDC-backbone minus LCC-backbone separation [95% CI]"],
        "rows": rows}


def collect(root):
    root = Path(root)
    V, T = {}, {}
    _validity(root, V, T)
    _stability(root, V, T)
    _books(root, V, T)
    _covered(root, V, T)
    _semantic(root, V, T)
    _external(root, V, T)
    return V, T


def md_table(t, align=None):
    h = t["header"]
    align = align or (["---"] + ["---:"] * (len(h) - 1))
    out = ["| " + " | ".join(h) + " |", "| " + " | ".join(align) + " |"]
    out += ["| " + " | ".join(str(c) for c in r) + " |" for r in t["rows"]]
    return "\n".join(out)


if __name__ == "__main__":
    import sys
    v, t = collect(Path(__file__).resolve().parents[2])
    for k, val in v.items():
        print(k, "=", val)
    for k, tab in t.items():
        print("\n##", k)
        print(md_table(tab))
    sys.exit(0)
