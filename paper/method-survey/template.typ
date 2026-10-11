// Classic two-column academic paper template (US Letter).
// Uses only fonts bundled with Typst (New Computer Modern, DejaVu Sans Mono)
// and no external packages, so it compiles offline.

#let body-font = "New Computer Modern"
#let mono-font = "DejaVu Sans Mono"
#let draft-text = "DRAFT, not submitted"

// Monospace identifier.
#let id(s) = raw(s)

// Table helpers. `rows` is an array of equal-length arrays of strings.
#let tbl(
  header,
  rows,
  align: auto,
  columns: auto,
  size: 7.2pt,
  mono-first: true,
  stripe: false,
) = {
  let n = header.len()
  let al = if align == auto {
    (left,) + (right,) * (n - 1)
  } else { align }
  let cols = if columns == auto { (auto,) * n } else { columns }
  set text(size: size)
  table(
    columns: cols,
    align: (x, y) => al.at(x),
    stroke: none,
    inset: (x: 3.2pt, y: 2.2pt),
    table.hline(stroke: 0.8pt),
    table.header(..header.map(h => strong(h)), repeat: true),
    table.hline(stroke: 0.4pt),
    ..rows
      .enumerate()
      .map(((i, r)) => r
        .enumerate()
        .map(((j, c)) => if j == 0 and mono-first { raw(c) } else { c }))
      .flatten(),
    table.hline(stroke: 0.8pt),
  )
}

// A table spanning both columns, floated to the top of a page.
#let wide-table(body, caption, label: none) = {
  let f = figure(
    body,
    caption: caption,
    kind: table,
    placement: top,
    scope: "parent",
  )
  if label == none { f } else { [#f #label] }
}

// A table inside one column.
#let col-table(body, caption, label: none) = {
  let f = figure(body, caption: caption, kind: table, placement: auto)
  if label == none { f } else { [#f #label] }
}

#let paper(
  title: [],
  short-title: [],
  authors: "AUTHOR TBD",
  abstract: [],
  keywords: (),
  body,
) = {
  set document(title: title, author: authors)
  set page(
    paper: "us-letter",
    margin: (x: 0.8in, top: 0.95in, bottom: 0.9in),
    columns: 2,
    header-ascent: 14pt,
    header: context {
      let p = counter(page).get().first()
      set text(size: 8pt)
      grid(
        columns: (1fr, auto),
        align: (left, right),
        if p > 1 { emph(short-title) } else { [] },
        strong(draft-text),
      )
      v(-4pt)
      line(length: 100%, stroke: 0.4pt)
    },
    footer: context {
      set text(size: 8.5pt)
      align(center, counter(page).display("1"))
    },
  )
  set columns(gutter: 0.28in)

  set text(font: body-font, size: 9.5pt, lang: "en", hyphenate: true)
  set par(justify: true, leading: 0.52em, spacing: 0.55em, first-line-indent: 1.1em)
  show raw: set text(font: mono-font, size: 0.82em)
  show link: set text(font: body-font)

  // Headings: "1 INTRODUCTION", "1.1 Title".
  set heading(numbering: "1.1", supplement: [Section])
  show heading.where(level: 1): it => {
    set text(size: 9.5pt, weight: "bold", hyphenate: false)
    block(above: 1.15em, below: 0.6em, sticky: true, {
      if it.numbering != none {
        counter(heading).display(it.numbering)
        h(0.7em)
      }
      upper(it.body)
    })
  }
  show heading.where(level: 2): it => {
    set text(size: 9.5pt, weight: "bold", hyphenate: false)
    block(above: 0.95em, below: 0.5em, sticky: true, {
      if it.numbering != none {
        counter(heading).display(it.numbering)
        h(0.6em)
      }
      it.body
    })
  }
  show heading.where(level: 3): it => {
    set text(size: 9.5pt, weight: "bold", style: "italic")
    block(above: 0.8em, below: 0.4em, sticky: true, it.body)
  }
  // The paragraph right after a heading is not indented (default), keep so.

  // Equations.
  set math.equation(numbering: "(1)", supplement: [Eq.])
  show math.equation: set text(font: "New Computer Modern Math")
  show math.equation.where(block: true): set block(above: 0.8em, below: 0.8em)

  // Figures and tables.
  set figure(gap: 0.55em)
  show figure.caption: set text(size: 8.3pt)
  show figure.caption: set par(justify: true, first-line-indent: 0em)
  show figure.caption: it => context {
    let n = it.counter.display(it.numbering)
    block(width: 100%, [*#it.supplement #n.* #it.body])
  }
  show figure.where(kind: table): set figure.caption(position: top)
  show figure.where(kind: table): set block(above: 0.8em, below: 0.9em)
  show figure: set block(above: 0.8em, below: 0.9em)

  // Lists.
  set list(indent: 0.8em, body-indent: 0.5em, spacing: 0.5em)
  set enum(indent: 0.8em, body-indent: 0.5em, spacing: 0.5em)

  // Title block, single column, spanning the page.
  place(
    top + center,
    float: true,
    scope: "parent",
    clearance: 1.4em,
    {
      set par(first-line-indent: 0em, justify: false)
      v(0.2em)
      align(center, {
        text(size: 16pt, weight: "bold", title)
        v(0.9em)
        text(size: 10.5pt, authors)
        v(0.25em)
        text(size: 9pt, emph[Working draft. #draft-text.])
      })
      v(0.9em)
      block(width: 100%, inset: (x: 0.35in), {
        set par(justify: true, first-line-indent: 0em, leading: 0.5em)
        set text(size: 9pt)
        align(center, strong[Abstract])
        v(0.15em)
        abstract
        v(0.4em)
        [*Keywords:* #keywords.join("; ")]
      })
      v(0.5em)
      line(length: 100%, stroke: 0.6pt)
    },
  )

  body
}

// Unnumbered heading helper (Acknowledgements, References).
#let unnumbered(t) = heading(numbering: none, level: 1, t)

// Scatter plot drawn with plain boxes: points = ((label, x, y), ...),
// x and y already scaled to [0, 1].
#let scatter(points, width: 100%, height: 5.2cm, xlabel: [], ylabel: [], xticks: (), yticks: ()) = {
  block(width: width, {
    set text(size: 7pt)
    let pad-l = 1.1cm
    let pad-b = 0.9cm
    box(width: 100%, height: height + pad-b, {
      // axes
      place(top + left, dx: pad-l, dy: 0pt, rect(width: 100% - pad-l - 3pt, height: height, stroke: 0.5pt))
      // y ticks
      for (lab, t) in yticks {
        place(top + left, dx: pad-l - 0.9cm, dy: height * (1 - t) - 3pt, box(width: 0.8cm, align(right, lab)))
        place(top + left, dx: pad-l, dy: height * (1 - t), line(length: 100% - pad-l - 3pt, stroke: 0.2pt + luma(200)))
      }
      for (lab, t) in xticks {
        place(top + left, dx: pad-l + (100% - pad-l - 3pt) * t - 0.5cm, dy: height + 2pt, box(width: 1cm, align(center, lab)))
      }
      place(top + left, dx: pad-l, dy: height + 0.42cm, box(width: 100% - pad-l - 3pt, align(center, xlabel)))
      place(top + left, dx: 0pt, dy: -0.35cm, ylabel)
      for (lab, x, y, anchor, dy) in points {
        let px = pad-l + (100% - pad-l - 3pt) * x
        place(top + left, dx: px - 1.6pt, dy: height * (1 - y) - 1.6pt, circle(radius: 1.6pt, fill: black))
        if anchor == "r" {
          place(top + left, dx: px + 3.5pt, dy: height * (1 - y) - 4pt + dy, raw(lab))
        } else {
          place(top + left, dx: px - 3.5pt - 2.1cm, dy: height * (1 - y) - 4pt + dy, box(width: 2.1cm, align(right, raw(lab))))
        }
      }
    })
  })
}
